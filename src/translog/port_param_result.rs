use crate::translog::{InsertParam, Transaction};

// Params

pub struct GetTransactionsParam {
    pub collection_id: String,
    pub from_op_seq: u64,
    pub count: u64,
}

pub struct InsertPortParam {
    pub collection_id: String,
    pub segment_id: String,
    pub op_seq: u64,
    pub transaction: Transaction,
}

// Results

pub struct InsertPortResult {
    pub byte_size: u64,
}

pub struct LoadOrCreateResult {
    pub segments: Vec<LoadOrCreateResultSegment>,
    pub last_committed_op_seq: u64,
    pub last_pending_op_seq: u64,
}

pub struct LoadOrCreateResultSegment {
    pub id: String,
    pub byte_size: u64,
    pub minimum_op_seq: u64,
}

pub struct GetTransactionsResult {
    pub last_op_seq: u64,
    pub transactions: Vec<Transaction>,
}
