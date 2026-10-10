use std::{sync::{Arc, atomic::AtomicU64}, thread, time::Duration};

use crate::{document::OpSeqCommitEvent, translog::{LoadOrCreate, GetTransactionsParam, GetTransactionsResult, InsertPortParam, LoadStateResult, Transaction, service_param_result::InsertParam, state::OpSeqState}, utils::{observer::observer::Observer, vixerr::Error, vixscheduler::Scheduler}};

pub trait TranslogService: Send + Sync {
    fn stream(&self, collection_id: &str) -> Result<Box<dyn TranslogStreamer>, Error>;
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
}

pub trait TranslogStreamer: Send + Sync {
    fn get_transactions(&mut self, count: u64) -> Result<Vec<Transaction>, Error>;
}

pub trait TranslogPort: Send + Sync {
    fn get_transactions(&self, param: GetTransactionsParam) -> Result<GetTransactionsResult, Error>;
    fn insert(&self, param: InsertPortParam) -> Result<(), Error>;
    fn commit(&self, op_seq: u64) -> Result<(), Error>;
    fn load_or_create(&self, collection_id: &str, segment_create_id: &str) -> Result<LoadOrCreate, Error>;
    fn unload(&self, collection_id: &str) -> Result<(), Error>;
}
