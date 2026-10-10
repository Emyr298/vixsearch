use crate::translog::{InsertParam, Transaction, state::SegmentState};

pub struct GetTransactionsParam {
    pub collection_id: String,
    pub from_op_seq: u64,
    pub count: u64,
}

pub type InsertPortParam = InsertParam;

pub struct LoadOrCreate {
    pub collection_id: String,
    pub committed_op_seq: u64,
    pub segments: Vec<LoadStateSegment>,
}

impl LoadOrCreate {
    pub fn segment_ids(&self) -> Vec<String> {
        self.segments.iter()
            .map(|s| s.id.clone())
            .collect()
    }
}

pub struct LoadStateSegment {
    pub id: String,
    pub minimum_op_seq: u64,
}

impl LoadStateSegment {
    pub fn segment_state(&self) -> SegmentState {
        SegmentState {
            id: self.id.clone(),
            minimum_op_seq: self.minimum_op_seq,
        }
    }
}

pub struct GetTransactionsResult {
    pub last_op_seq: u64,
    pub transactions: Vec<Transaction>,
}
