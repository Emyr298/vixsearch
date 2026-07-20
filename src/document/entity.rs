use std::{collections::HashMap, fmt::{Display, Formatter, Result}};

use serde::{Deserialize, Serialize};

use crate::shared::Value;

pub fn key_from_id(id: &str) -> String {
    format!("id_{}", id)
}

pub fn key_from_seq_id(seq_id: &str) -> String {
    format!("seq_{}", seq_id)
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
