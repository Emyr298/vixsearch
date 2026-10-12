use std::sync::Arc;

use crate::{document::OpSeqCommitEvent, errcode::PARSE_ERROR, storage::{OUT_OF_RANGE, Storage}, translog::{GetTransactionsParam, GetTransactionsResult, InsertPortParam, LoadOrCreateResult, LoadOrCreateResultSegment, TranslogPort, adapter::helper::{Commit, CommitBlock, GetLastCommitResult, HEADER_LEN, SegmentBlockHeader, SegmentBlockTransaction, commit_store, empty_load_or_create_result, latest_commit_name, next_commit_number, read_transaction_log, segment_name, segment_store}}, utils::{observer::observer::Observer, vixerr::Error}};

/*

TODO:
- cleanup mechanism
- invalid commits & old commits should be cleaned at load (another cleanup mechanism)
- commit should be done rarely, like every 2-5 seconds

*/

pub struct TranslogAdapter {
    storage: Arc<dyn Storage>,
    max_transaction_len: u64,
}

impl TranslogAdapter {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        TranslogAdapter {
            storage,
        }
    }
}

impl TranslogAdapter {
    fn create_segment(&self) {

    }

    // first log in a segment must be valid before commit
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
        let mut last_appended_op_seq: u64 = 0;
        for i in 0..last_commit.segment_ids.len() {
            let segment_id = &last_commit.segment_ids[i];

            let reader = self.storage.get_reader(&segment_store(collection_id), &segment_name(segment_id))?;
            let (mut transaction, mut next_offset) = read_transaction_log(&reader, 0, collection_id, self.max_transaction_len)?;

            segments.push(LoadOrCreateResultSegment {
                id: segment_id.to_string(),
                minimum_op_seq: transaction.op_seq,
            });

            // active segment
            if i == last_commit.segment_ids.len() - 1 {
                loop {
                    match read_transaction_log(&reader, next_offset, collection_id, self.max_transaction_len) {
                        Ok((t, o)) => {
                            transaction = t;
                            next_offset = o;
                        },
                        Err(e) if e.code == PARSE_ERROR || e.code == OUT_OF_RANGE => {
                            break
                        },
                        Err(e) => return Err(e),
                    };
                }

                last_appended_op_seq = transaction.op_seq;
                active_segment_next_offset = next_offset;
                active_segment_byte_size = reader.byte_size()?;
            }
        }

        Ok(LoadOrCreateResult {
            active_segment_next_offset: active_segment_next_offset,
            active_segment_byte_size: active_segment_byte_size,
            segments,
            last_durable_op_seq: last_commit.last_durable_op_seq,
            last_appended_op_seq: last_appended_op_seq,
        })
    }

    fn get_transactions(&self, param: GetTransactionsParam) -> Result<GetTransactionsResult, Error> {
        todo!()
    }

    fn insert(&self, param: InsertPortParam) -> Result<crate::translog::InsertPortResult, Error> {
        todo!()
    }
    
    fn commit_op_seq(&self, op_seq: u64) -> Result<(), Error> {
        todo!()
    }
    
    fn unload(&self, collection_id: &str, segment_id: &str) -> Result<(), Error> {
        todo!()
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
        let reader = self.storage.get_reader(commit_store, commit_name)?;
        let commit_buf = reader.read_all()?;

        let commit_block = CommitBlock::from_buf(commit_buf)?;
        commit_block.decode()
    }
}
