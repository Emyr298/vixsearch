use std::collections::HashMap;

use crate::translog::{InsertPortParam, entity::Transaction};

pub struct InsertParam {
    pub collection_id: String,
    pub op_seq: u64,
    pub transaction: Transaction,
}

impl InsertParam {
    pub fn insert_port_param(self, segment_id: &str, segment_next_offset: u64) -> InsertPortParam {
        InsertPortParam {
            collection_id: self.collection_id,
            segment_id: segment_id.to_string(),
            op_seq: self.op_seq,
            transaction: self.transaction,
            segment_next_offset,
        }
    }
}

pub struct StreamTrxLogsParam {
    pub collection_id: String,
    pub start_op_seq: u64,
    pub is_continuous: bool,
    pub max_batch: usize,
    pub max_outstanding: usize,
}
