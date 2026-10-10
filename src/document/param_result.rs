use std::collections::HashMap;

use crate::common::{Document, Value};

pub struct InsertParam {
    pub doc_id: String,
    pub op_seq: u64,
    pub payload: HashMap<String, Value>,
}

impl InsertParam {
    pub fn document(self, existing_seq_id: Option<u64>) -> Document {
        Document {
            id: self.doc_id,
            seq_id: match existing_seq_id {
                Some(s) => s,
                None => self.op_seq,
            },
            payload: self.payload,
        }
    }
}
