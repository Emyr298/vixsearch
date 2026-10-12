use std::collections::HashMap;
use std::sync::Arc;

use deku::prelude::*;
use deku::{DekuRead, DekuWrite};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::Value;
use crate::storage::StorageReader;
use crate::translog::{InsertDetail, LoadOrCreateResult, LoadOrCreateResultSegment, Transaction, TransactionDetail};
use crate::{errcode::PARSE_ERROR, utils::vixerr::Error};

// Commit

pub const COMMIT_MAGIC: &[u8; 4] = b"CMMT";

pub fn commit_store(collection_id: &str) -> String {
    format!("collection/{}/commit", collection_id)
}

pub struct GetLastCommitResult {
    pub next_number: u64,
    pub commit: Option<Commit>,
}

impl GetLastCommitResult {
    pub fn new(next_number: u64, commit: Option<Commit>) -> Self {
        GetLastCommitResult {
            next_number,
            commit,
        }
    }
}

pub fn next_commit_number(names: &[String]) -> u64 {
    for name in names.iter().rev() {
        let Some(hex) = name.strip_prefix("commit_") else { continue };

        if hex.len() == 16 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let num = u64::from_str_radix(hex, 16).unwrap();
            return num.checked_add(1).unwrap();
        }
    }

    1
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub commit_number: u64,
    pub last_durable_op_seq: u64,
    pub segment_ids: Vec<String>,
}

impl Commit {
    pub fn name(&self) -> String {
        format!("commit_{:016x}", self.commit_number)
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct CommitBlock {
    magic: [u8; 4],
    hash: [u8; 4],
    data_len: u64,
    #[deku(count = "data_len")]
    data: Vec<u8>,
}

impl CommitBlock {
    pub fn from_commit(commit: Commit) -> Self {
        let commit_buf = rmp_serde::to_vec(&commit).unwrap();

        CommitBlock {
            magic: *COMMIT_MAGIC,
            hash: crc32fast::hash(&commit_buf).to_le_bytes(),
            data_len: (commit_buf.len() as u64),
            data: commit_buf,
        }
    }

    pub fn from_buf(buf: Vec<u8>) -> Result<Self, Error> {
        let (_, block) = match CommitBlock::from_bytes((&buf, 0)) {
            Ok(b) => b,
            Err(e) => return Error::code(PARSE_ERROR)
                .message("failed to decode commit buffer")
                .wrap(e)
                .throw(),
        };

        let expected_hash = u32::from_le_bytes(block.hash.try_into().unwrap());
        let actual_hash = crc32fast::hash(&block.data);
        if expected_hash != actual_hash {
            return Error::code(PARSE_ERROR)
                .message("invalid commit hash")
                .throw();
        }
        
        Ok(block)
    }

    pub fn encode(self) -> Vec<u8> {
        self.to_bytes().unwrap()
    }

    pub fn decode(self) -> Result<Commit, Error> {
        let commit: Commit = match rmp_serde::from_slice(&self.data) {
            Ok(c) => c,
            Err(e) => return Error::code(PARSE_ERROR)
                .message("failed to decode commit buffer")
                .wrap(e)
                .throw(),
        };

        Ok(commit)
    }
}

pub fn empty_load_or_create_result() -> LoadOrCreateResult {
    LoadOrCreateResult{
        active_segment_next_offset: 0,
        active_segment_byte_size: 0,
        segments: vec![LoadOrCreateResultSegment{
            id: Uuid::new_v4().to_string(),
            minimum_op_seq: 0,
        }],
        last_durable_op_seq: 0,
        last_appended_op_seq: 0,
    }
}

// Segment

pub const BLOCK_MAGIC: &[u8; 4] = b"BLCK";

pub fn segment_store(collection_id: &str) -> String {
    format!("collection/{}/translog", collection_id)
}

pub fn segment_name(segment_id: &str) -> String {
    format!("translog_{}", segment_id)
}

pub const HEADER_LEN: u64 = 16;

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SegmentBlockHeader {
    magic: [u8; 4],
    transaction_hash: [u8; 4],
    pub transaction_len: u64,
}

impl SegmentBlockHeader {
    pub fn from_buf(buf: Vec<u8>, max_transaction_len: u64) -> Result<Self, Error> {
        let (_, header) = match SegmentBlockHeader::from_bytes((&buf, 0)) {
            Ok(b) => b,
            Err(e) => return Error::code(PARSE_ERROR)
                .message("failed to decode segment block buffer")
                .wrap(e)
                .throw(),
        };

        if header.magic != *BLOCK_MAGIC {
            return Error::code(PARSE_ERROR)
                .message(format!("invalid magic"))
                .throw()
        }

        // TODO: dont forget to use hash later
        if header.transaction_len > max_transaction_len {
            return Error::code(PARSE_ERROR)
                .message("block transaction length is more than maximum")
                .throw()
        }
        
        Ok(header)
    }

    pub fn validate_transaction(&self, transaction_buf: &[u8]) -> Result<(), Error> {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&self.transaction_len.to_le_bytes());
        hasher.update(transaction_buf);

        let actual_hash = hasher.finalize();
        let expected_hash = u32::from_le_bytes(self.transaction_hash.try_into().unwrap());

        if actual_hash != expected_hash {
            return Error::code(PARSE_ERROR)
                .message("invalid segment transaction hash")
                .throw();
        }

        return Ok(())
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SegmentBlockTransaction {
    transaction_type: [u8; 4],
    pub op_seq: u64,
    #[deku(read_all)]
    detail: Vec<u8>,
}

impl SegmentBlockTransaction {
    pub fn from_buf(buf: Vec<u8>) -> Result<Self, Error> {
        let (_, transaction) = match SegmentBlockTransaction::from_bytes((&buf, 0)) {
            Ok(b) => b,
            Err(e) => return Error::code(PARSE_ERROR)
                .message("failed to decode segment transaction buffer")
                .wrap(e)
                .throw(),
        };
        Ok(transaction)
    }

    pub fn detail(&self) -> Result<SegmentBlockTransactionDetail, Error> {
        match &self.transaction_type {
            b"INST" => {
                let detail: SegmentBlockTransactionDetailInsert = match rmp_serde::from_slice(&self.detail) {
                    Ok(r) => r,
                    Err(e) => return Error::code(PARSE_ERROR)
                        .message("failed to decode segment transaction detail")
                        .wrap(e)
                        .throw(),
                };
                Ok(SegmentBlockTransactionDetail::Insert(detail))
            },
            _ => Error::code(PARSE_ERROR)
                .message("invalid transaction type")
                .throw(),
        }
    }
}

pub enum SegmentBlockTransactionDetail {
    Insert(SegmentBlockTransactionDetailInsert),
}

impl SegmentBlockTransactionDetail {
    pub fn transaction_detail(self) -> TransactionDetail {
        match self {
            SegmentBlockTransactionDetail::Insert(d) => TransactionDetail::Insert(d.insert_detail()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SegmentBlockTransactionDetailInsert {
    pub document_id: String,
    pub payload: HashMap<String, Value>
}

impl SegmentBlockTransactionDetailInsert {
    pub fn insert_detail(self) -> InsertDetail {
        InsertDetail {
            document_id: self.document_id,
            payload: self.payload,
        }
    }
}

pub fn read_transaction_log(reader: &Box<dyn StorageReader>, offset: u64, collection_id: &str, max_transaction_len: u64) -> Result<(Transaction, u64), Error> {
    let header_buf = reader.read(offset, HEADER_LEN)?;
    let block_header = SegmentBlockHeader::from_buf(header_buf, max_transaction_len)?;

    let transaction_buf = reader.read(offset + HEADER_LEN, block_header.transaction_len)?;

    block_header.validate_transaction(&transaction_buf)?;

    let transaction = SegmentBlockTransaction::from_buf(transaction_buf)?;
    let transaction_detail = transaction.detail()?;

    Ok((
        Transaction {
            collection_id: collection_id.to_string(),
            op_seq: transaction.op_seq,
            detail: transaction_detail.transaction_detail(),
        },
        offset + HEADER_LEN + block_header.transaction_len
    ))
}
