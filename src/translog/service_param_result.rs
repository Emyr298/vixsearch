use std::collections::HashMap;

use crate::translog::entity::Transaction;

pub struct InsertParam {
    pub collection_id: String,
    pub op_seq: u64,
    pub transaction: Transaction,
}

pub struct LoadStateResult {
    pub committed_op_seq_by_collection_id: HashMap<String, u64>,
}
