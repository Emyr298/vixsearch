use std::sync::{Arc, Mutex};

use dashmap::DashMap;

use crate::{document::OpSeqCommitEvent, errcode::PARSE_ERROR, storage::{OUT_OF_RANGE, Storage}, translog::{GetTransactionsParam, GetTransactionsResult, InsertPortParam, InsertPortResult, InsertPortResultSegment, LoadOrCreateResult, LoadOrCreateResultSegment, Transaction, TranslogPort, adapter::adapter_helper::{Commit, CommitBlock, GetLastCommitResult, HEADER_LEN, SegmentBlockHeader, SegmentBlockTransaction, commit_store, create_trx_log_buffer, empty_load_or_create_result, next_commit_number, read_transaction_log, segment_name, segment_store, write_trx_log_buffer}}, utils::{locker::keyed::KeyedLockManager, observer::observer::Observer, vixerr::Error}};

/*

TODO:
- cleanup mechanism
- invalid commits & old commits should be cleaned at load (another cleanup mechanism)
- commit should be done rarely, like every 2-5 seconds

*/

pub struct TranslogAdapter {
    storage: Arc<dyn Storage>,
    max_transaction_len: u64,
    max_segment_byte_size: u64,
}

impl TranslogAdapter {
    pub fn new(storage: Arc<dyn Storage>, max_segment_byte_size: u64, max_transaction_len: u64) -> Self {
        TranslogAdapter {
            storage,
            max_transaction_len,
            max_segment_byte_size,
        }
    }
}

impl TranslogPort for TranslogAdapter {
    fn load_or_create(&self, collection_id: &str) -> Result<LoadOrCreateResult, Error> {
        let last_commit_result = self.get_last_commit(collection_id)?;

        let last_commit = match last_commit_result.commit { // commit should be done when atleast one segment & one log
            Some(c) => c,
            None => return Ok(empty_load_or_create_result()), // TODO: should we create file on insert or from zero?
        };

        let mut segments: Vec<LoadOrCreateResultSegment> = Vec::new();
        
        let mut active_segment_next_offset: u64 = 0;
        let mut active_segment_byte_size: u64 = 0;
        for i in 0..last_commit.segment_ids.len() {
            let segment_id = &last_commit.segment_ids[i];

            let reader = self.storage.get_or_create_reader(&segment_store(collection_id), &segment_name(segment_id))?;
            let (mut transaction, mut next_offset) = read_transaction_log(&reader, 0, collection_id, self.max_transaction_len)?;

            segments.push(LoadOrCreateResultSegment {
                id: segment_id.to_string(),
                minimum_op_seq: transaction.op_seq,
            });

            // active segment
            if i == last_commit.segment_ids.len() - 1 {
                loop {
                    match read_transaction_log(&reader, next_offset, collection_id, self.max_transaction_len) {
                        Ok((_, o)) => {
                            next_offset = o;
                        },
                        Err(e) if e.code == PARSE_ERROR || e.code == OUT_OF_RANGE => {
                            break
                        },
                        Err(e) => return Err(e),
                    };
                }

                active_segment_next_offset = next_offset;
                active_segment_byte_size = reader.byte_size()?;
            }
        }

        Ok(LoadOrCreateResult {
            active_segment_next_offset: active_segment_next_offset,
            active_segment_byte_size: active_segment_byte_size,
            segments,
        })
    }

    fn get_transactions(&self, param: GetTransactionsParam) -> Result<GetTransactionsResult, Error> {
        let reader = self.storage.get_or_create_reader(&segment_store(&param.collection_id), &segment_name(&param.segment_id))?;

        let mut block_offset = param.from_block_offset;

        let mut trxs: Vec<Transaction> = Vec::new();
        loop {
            let (trx, next_offset) = match read_transaction_log(&reader, block_offset, &param.collection_id, self.max_transaction_len) {
                Ok((t, o)) => (t, o),
                Err(e) if e.code == PARSE_ERROR || e.code == OUT_OF_RANGE => break, // block offset given should be right, otherwise PARSE_ERROR could mean shifted offset, not end of valid file
                Err(e) => return Err(e),
            };
            trxs.push(trx);
            block_offset = next_offset;
        }

        Ok(GetTransactionsResult {
            transactions: trxs,
            next_block_offset: block_offset,
        })
    }

    fn insert(&self, param: InsertPortParam) -> Result<InsertPortResult, Error> {
        let trx_log_buffer = create_trx_log_buffer(&param.collection_id, param.transaction)?;

        let mut segment_id = param.segment_id.clone();
        if param.segment_byte_size + trx_log_buffer.len() as u64 > self.max_segment_byte_size {
            segment_id = uuid::Uuid::new_v4().to_string();
        }

        let writer = self.storage.get_or_create_random_access_writer(
            &segment_store(&param.collection_id),
            &segment_name(&segment_id),
        )?;

        writer.write(param.segment_next_offset, &trx_log_buffer)?;
        writer.flush()?;

        // TODO: it's only for add segment case
        let last_commit_result = self.get_last_commit(&param.collection_id)?;
        let last_commit_opt = last_commit_result.commit;
        let next_commit_number = last_commit_result.next_number;

        let cur_commit = Commit::next(last_commit_opt, &segment_id, next_commit_number);
        let cur_commit_name = cur_commit.name();
        
        let commit_block = CommitBlock::from_commit(cur_commit);
        let commit_buf = commit_block.to_buf();

        let mut commit_writer = self.storage.create_write_once_writer(
            &commit_store(&param.collection_id),
            &cur_commit_name,
        )?;

        commit_writer.write(&commit_buf)?;
        commit_writer.commit()?;

        Ok(InsertPortResult {
            active_segment_next_offset: param.segment_next_offset + trx_log_buffer.len() as u64,
            active_segment_byte_size: param.segment_byte_size + trx_log_buffer.len() as u64,
            added_segment: (param.segment_id != segment_id).then_some(InsertPortResultSegment {
                id: segment_id,
                minimum_op_seq: param.op_seq,
            }),
        })
    }

    fn unload(&self, collection_id: &str, segment_id: &str) {
        self.storage.close(&segment_store(collection_id), &segment_name(segment_id));
    }
}

impl TranslogAdapter {
    fn get_last_commit(&self, collection_id: &str) -> Result<GetLastCommitResult, Error> {
        let commit_store = commit_store(collection_id);
        let names = self.storage.get_all_name_sorted(&commit_store)?;
        let next_commit_number = next_commit_number(&names);

        for commit_name in names.into_iter().filter(|n| n.starts_with("commit_")).rev() {
            match self.get_commit(&commit_store, &commit_name) {
                Ok(c) => return Ok(GetLastCommitResult::new(next_commit_number, Some(c))),
                Err(e) if e.code != PARSE_ERROR => return Err(e),
                Err(_) => continue,
            }
        }

        Ok(GetLastCommitResult::new(next_commit_number, None))
    }

    fn get_commit(&self, commit_store: &str, commit_name: &str) -> Result<Commit, Error> {
        let reader = self.storage.get_or_create_reader(commit_store, commit_name)?;
        let commit_buf = reader.read_all()?;

        let commit_block = CommitBlock::from_buf(commit_buf)?;
        commit_block.decode()
    }
}
