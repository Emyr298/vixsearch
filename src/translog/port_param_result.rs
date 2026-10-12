use crate::translog::{InsertParam, Transaction};

// Params

pub struct GetTransactionsParam {
    pub collection_id: String,
    pub segment_id: String,
    pub from_op_seq: u64,
    pub max_count: usize,
}

pub struct InsertPortParam {
    pub collection_id: String,
    pub segment_id: String,
    pub op_seq: u64,
    pub transaction: Transaction,
    pub segment_next_offset: u64,
}

// Results

pub struct InsertPortResult {
    pub active_segment_next_offset: u64,
    pub active_segment_byte_size: u64,
    pub added_segment: Option<InsertPortResultSegment>,
}

pub struct InsertPortResultSegment {
    pub id: String,
    pub minimum_op_seq: u64,
}

pub struct LoadOrCreateResult {
    pub active_segment_byte_size: u64,
    pub active_segment_next_offset: u64,
    pub segments: Vec<LoadOrCreateResultSegment>,

    pub last_durable_op_seq: u64,
    pub last_appended_op_seq: u64,
}

pub struct LoadOrCreateResultSegment {
    pub id: String,
    pub minimum_op_seq: u64,
}

pub struct GetTransactionsResult {
    pub last_op_seq: u64,
    pub transactions: Vec<Transaction>,
}
