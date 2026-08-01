use std::{collections::HashMap, fmt::{Display, Formatter, Result}};

use serde::{Deserialize, Serialize};

use crate::shared::Value;

pub fn key_from_id(id: &str) -> Vec<u8> {
    format!("id_{}", id).into_bytes()
}

pub fn key_from_seq_id(seq_id: &u64) -> Vec<u8> {
    [b"seq_", &seq_id.to_le_bytes()[..]].concat()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub seq_id: String,
    pub payload: HashMap<String, Value>,
}

impl Display for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self)
    }
}
