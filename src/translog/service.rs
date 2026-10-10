use std::{sync::{Arc, atomic::AtomicU64}, thread, time::Duration};

use crate::{document::OpSeqCommitEvent, translog::{GetTransactionsParam, GetTransactionsResult, InsertPortParam, InsertPortResult, LoadOrCreateResult, LoadStateResult, Transaction, service_param_result::InsertParam, state::CollectionState}, utils::{observer::observer::Observer, vixerr::Error, vixscheduler::Scheduler}};

pub trait TranslogService: Send + Sync {
    fn stream_uncommitted_transactions(&self, collection_id: &str) -> Result<Box<dyn TranslogStreamer>, Error>;
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
}

pub trait TranslogStreamer: Send + Sync {
    fn get_transactions(&mut self, count: u64) -> Result<Vec<Transaction>, Error>;
}

pub trait TranslogPort: Send + Sync {
    fn get_transactions(&self, param: GetTransactionsParam) -> Result<GetTransactionsResult, Error>;
    fn insert(&self, param: InsertPortParam) -> Result<InsertPortResult, Error>;
    fn load_or_create(&self, collection_id: &str) -> Result<LoadOrCreateResult, Error>;
    fn commit_op_seq(&self, op_seq: u64) -> Result<(), Error>;
    // fn load_or_create(&self, collection_id: &str, segment_create_id: &str) -> Result<LoadOrCreateResult, Error>;
    // fn unload(&self, collection_id: &str) -> Result<(), Error>;
}
