use std::collections::HashMap;

use crate::document::Value;

pub enum Transaction {
    Insert(InsertData),
}

pub struct InsertData {
    pub document_id: String,
    pub payload: HashMap<String, Value>
}
