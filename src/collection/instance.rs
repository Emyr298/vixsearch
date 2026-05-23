use std::collections::HashMap;

use crate::shared;

pub struct CollectionInstance {
    pub id: String,
    pub name: String,
    pub field_types: HashMap<String, shared::ValueType>,
}

impl CollectionInstance {
    pub fn validate_document(&self, document: &HashMap<String, shared::Value>) -> bool {
        for (key, value) in document {
            if !self.field_types.contains_key(key) {
                return false;
            }

            if value.value_type() != self.field_types[key] {
                return false;
            }
        }
        true
    }
}
