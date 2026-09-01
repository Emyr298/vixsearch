use std::collections::HashMap;

use crate::document::{Document, Value};

pub struct InsertParam {
    pub id: String,
    pub op_seq: u64,
    pub payload: HashMap<String, Value>,
}

impl InsertParam {
    pub fn document(self) -> Document {
        Document {
            id: self.id,
            seq_id: self.op_seq,
            payload: self.payload,
        }
    }
}
