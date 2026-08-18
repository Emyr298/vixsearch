use deku::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{collection::port_param_result::{CreatePortParam, CreatePortParamField, GetAllPortResultCollection, GetAllPortResultCollectionField}, document::ValueType, errcode::PARSE_ERROR, utils::vixerr::Error};

pub const COMMIT_MAGIC: &[u8; 4] = b"CMMT";
pub const SCHEMA_MAGIC: &[u8; 4] = b"SCMA";

pub const COMMIT_STORE: &str = "collection/commit";

// TODO: always up the version in next phase when alter collection exists
pub fn schema_store_name(collection_internal_id: &str) -> (String, String) {
    (format!("collection/{}/schema", collection_internal_id), format!("schema_{:016x}", 1))
}

pub struct GetLatestCommitResult {
    pub next_number: u64,
    pub latest_commit: Option<Commit>,
}

impl GetLatestCommitResult {
    pub fn new(next_number: u64, latest_commit: Option<Commit>) -> Self {
        GetLatestCommitResult {
            next_number,
            latest_commit,
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
pub struct Schema {
    pub id: String,
    pub internal_id: String,
    pub fields: Vec<SchemaField>,
}

impl Schema {
    pub fn new(param: CreatePortParam) -> Self {
        Self {
            id: param.id,
            internal_id: param.internal_id,
            fields: param.fields.into_iter()
                .map(|f| SchemaField::new(f))
                .collect(),
        }
    }

    pub fn get_all_port_result_collection(self) -> Result<GetAllPortResultCollection, Error> {
        Ok(GetAllPortResultCollection {
            id: self.id,
            internal_id: self.internal_id,
            fields: self.fields.into_iter()
                .map(|f| f.get_all_port_result_collection_field())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String,
}

impl SchemaField {
    pub fn new(param: CreatePortParamField) -> Self {
        Self {
            name: param.name,
            field_type: param.field_type.string(),
        }
    }

    pub fn get_all_port_result_collection_field(self) -> Result<GetAllPortResultCollectionField, Error> {
        Ok(GetAllPortResultCollectionField {
            name: self.name,
            field_type: ValueType::new(&self.field_type)?,
        })
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct SchemaBlock {
    magic: [u8; 4],
    hash: [u8; 4],
    schema_len: u64,
    #[deku(count = "schema_len")]
    schema: Vec<u8>,
}

impl SchemaBlock {
    pub fn from_create_port_param(param: CreatePortParam) -> Self {
        let schema = Schema::new(param);
        let schema_buf = rmp_serde::to_vec(&schema).unwrap();

        SchemaBlock {
            magic: *SCHEMA_MAGIC,
            hash: crc32fast::hash(&schema_buf).to_le_bytes(),
            schema_len: (schema_buf.len() as u64),
            schema: schema_buf,
        }
    }

    pub fn from_buf(buf: Vec<u8>) -> Result<Self, Error> {
        let (_, block) = match SchemaBlock::from_bytes((&buf, 0)) {
            Ok(b) => b,
            Err(e) => return Error::code(PARSE_ERROR)
                .message("failed to decode schema buffer")
                .wrap(e)
                .throw(),
        };

        let expected_hash = u32::from_le_bytes(block.hash.try_into().unwrap());
        let actual_hash = crc32fast::hash(&block.schema);
        if expected_hash != actual_hash {
            return Error::code(PARSE_ERROR)
                .message("invalid schema hash")
                .throw();
        }
        
        Ok(block)
    }

    pub fn encode(self) -> Vec<u8> {
        self.to_bytes().unwrap()
    }

    pub fn decode(self) -> Result<Schema, Error> {
        let schema: Schema = match rmp_serde::from_slice(&self.schema) {
            Ok(c) => c,
            Err(e) => return Error::code(PARSE_ERROR)
                .message("failed to decode schema buffer")
                .wrap(e)
                .throw(),
        };

        Ok(schema)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub commit_number: u64,
    pub collection_internal_ids: Vec<String>,
}

impl Commit {
    pub fn add_collection(internal_id: &str, result: GetLatestCommitResult) -> Self {
        match result.latest_commit {
            Some(mut c) => {
                c.collection_internal_ids.push(internal_id.to_string());
                Commit {
                    commit_number: result.next_number,
                    collection_internal_ids: c.collection_internal_ids,
                }
            },
            None => Commit {
                commit_number: result.next_number,
                collection_internal_ids: vec![internal_id.to_string()],
            },
        }
    }

    pub fn delete_collection(internal_id: &str, result: GetLatestCommitResult) -> Option<Self> {
        match result.latest_commit {
            Some(mut c) => {
                c.collection_internal_ids.retain(|iid| iid != internal_id);
                Some(Commit {
                    commit_number: result.next_number,
                    collection_internal_ids: c.collection_internal_ids,
                })
            },
            None => None,
        }
    }

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
