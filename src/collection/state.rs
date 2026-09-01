use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::{collection::{port_param_result::GetAllPortResultCollection, service_param_result::CreateParamField}, document::{Document, RawValue, Value, ValueType}, errcode::PARSE_ERROR, utils::vixerr::Error};

#[derive(Debug, PartialEq)]
pub enum CollectionStatus {
    Loading,
    Ready,
    Deleting,
}

pub struct CollectionState {
    pub id: String,
    pub internal_id: String,
    pub status: CollectionStatus,
    pub field_types: HashMap<String, ValueType>,
}

impl CollectionState {
    pub fn new(id: &str, internal_id: &str, fields: &Vec<CreateParamField>, status: CollectionStatus) -> Self {
        CollectionState {
            id: id.to_string(),
            internal_id: internal_id.to_string(),
            status: status,
            field_types: fields.iter()
                .map(|field| (field.name.clone(), field.field_type.clone()))
                .collect(),
        }
    }

    pub fn from_get_all_port_result_collection(result: &GetAllPortResultCollection, status: CollectionStatus) -> Self {
        CollectionState {
            id: result.id.clone(),
            internal_id: result.internal_id.clone(),
            status,
            field_types: result.fields.iter()
                .map(|field| (field.name.clone(), field.field_type.clone()))
                .collect(),
        }
    }

    // TODO: optional constraint, etc in next phases
    pub fn parse_raw_document_payload(&self, raw_document_payload: &HashMap<String, RawValue>) -> Result<HashMap<String, Value>, Error> {
        let mut document_payload = HashMap::<String, Value>::new();

        for (key, raw_value) in raw_document_payload {
            let field_type = match self.field_types.get(key) {
                Some(ft) => ft,
                None => return Error::code(PARSE_ERROR)
                    .message(format!("unknown field: {}", key))
                    .throw(),
            };

            let value = field_type.value(&raw_value)?;
            document_payload.insert(key.clone(), value);
        }

        if document_payload.len() < self.field_types.len() {
            return Error::code(PARSE_ERROR)
                .message("missing fields")
                .throw();
        }

        Ok(document_payload)
    }
}

pub fn collection_ids(collection_states: &Vec<(String, String, Arc<RwLock<CollectionState>>)>) -> Vec<String> {
    collection_states.iter()
        .map(|cs| cs.1.clone())
        .collect()
}
