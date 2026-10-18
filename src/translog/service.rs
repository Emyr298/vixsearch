use std::{sync::{Arc, atomic::AtomicU64}, thread, time::Duration};

use crate::{document::OpSeqCommitEvent, translog::{GetTransactionsParam, GetTransactionsResult, InsertPortParam, InsertPortResult, LoadOrCreateResult, StreamTrxLogsParam, Transaction, service_param_result::InsertParam, state::CollectionState}, utils::{observer::observer::Observer, vixerr::Error, vixscheduler::Scheduler}};

// Translog

pub trait TranslogService: Send + Sync {
    fn stream_trx_logs(&self, param: StreamTrxLogsParam) -> Result<Box<dyn TranslogStreamer>, Error>;
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
}

pub trait TranslogStreamer: Send {
    fn recv_trx_log(&mut self, max_count: u64) -> Result<Transaction, Error>;
}

pub trait TranslogPort: Send + Sync {
    fn get_transactions(&self, param: GetTransactionsParam) -> Result<GetTransactionsResult, Error>;
    fn insert(&self, param: InsertPortParam) -> Result<InsertPortResult, Error>;

    fn load_or_create(&self, collection_id: &str) -> Result<LoadOrCreateResult, Error>;
    fn unload(&self, collection_id: &str, segment_id: &str);
}

// Checkpoint

pub trait CheckpointService: Send + Sync {
    fn get_checkpoint_op_seq(&self, collection_id: &str) -> Result<u64, Error>;
}

pub trait CheckpointPort: Send + Sync {
    fn get_checkpoint_op_seq(&self, collection_id: &str) -> Result<u64, Error>;
    fn submit_checkpoint(&self, op_seq: u64) -> Result<(), Error>;
}
