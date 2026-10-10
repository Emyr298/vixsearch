use std::collections::HashMap;

use crate::common::Value;

pub struct Transaction {
    pub collection_id: String,
    pub op_seq: u64,
    pub detail: TransactionDetail,
}

pub enum TransactionDetail {
    Insert(InsertDetail),
}

pub struct InsertDetail {
    pub document_id: String,
    pub payload: HashMap<String, Value>
}

pub struct OpSeqStateDropEvent {
    pub collection_id: String,
}
