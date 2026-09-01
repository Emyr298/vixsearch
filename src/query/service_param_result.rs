use std::collections::HashMap;

use crate::{document::{self, Document, RawValue, Value}, translog::{self, InsertData, Transaction}};

pub struct InsertParam {
    pub collection_id: String,
    pub document_id: String,
    pub payload: HashMap<String, RawValue>,
}

pub struct ParsedInsertParam {
    pub collection_id: String,
    pub document_id: String,
    pub payload: HashMap<String, Value>,
}

impl ParsedInsertParam {
    pub fn document_insert_param(self, op_seq: u64) -> document::InsertParam {
        document::InsertParam {
            id: self.document_id,
            op_seq,
            payload: self.payload,
        }
    }

    pub fn translog_insert_param(&self, op_seq: u64) -> translog::InsertParam {
        translog::InsertParam {
            op_seq,
            collection_id: self.collection_id.clone(),
            transaction: Transaction::Insert(InsertData {
                document_id: self.document_id.clone(),
                payload: self.payload.clone(),
            })
        }
    }
}
