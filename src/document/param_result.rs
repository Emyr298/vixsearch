use std::collections::HashMap;

use crate::shared::Value;

pub struct InsertParam {
    pub id: String,
    pub op_seq: u64,
    pub payload: HashMap<String, Value>,
}
