use std::collections::HashMap;

use crate::{collection::Field, shared};

#[derive(Debug, PartialEq)]
pub enum InstanceStatus {
    Loading,
    Ready,
}

pub struct CollectionInstance {
    pub id: String,
    pub name: String,
    pub status: InstanceStatus,
    pub field_types: HashMap<String, shared::ValueType>,
}

impl CollectionInstance {
    pub fn new(id: &str, name: &str, fields: &Vec<Field>, status: InstanceStatus) -> Self {
        let mut field_types = HashMap::new();
        for field in fields {
            field_types.insert(field.name.clone(), field.field_type.clone());
        }

        CollectionInstance {
            id: id.to_string(),
            name: name.to_string(),
            status: status,
            field_types,
        }
    }

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
