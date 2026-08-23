use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::{collection::{port_param_result::GetAllPortResultCollection, service_param_result::CreateParamField}, document::{Document, ValueType}, errcode::PARSE_ERROR, utils::vixerr::Error};

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
    pub fn validate_document(&self, document: &Document) -> Result<(), Error> {
        for (key, value) in &document.payload {
            if !self.field_types.contains_key(key) {
                return Error::code(PARSE_ERROR)
                    .message(format!("unknown field: {}", key))
                    .throw();
            }

            let payload_type = value.value_type();
            let collection_type = &self.field_types[key];
            if &payload_type != collection_type {
                return Error::code(PARSE_ERROR)
                    .message(format!("field {} is not {}: {}", key, payload_type.string(), collection_type.string()))
                    .throw();
            }
        }

        if document.payload.len() < self.field_types.len() {
            return Error::code(PARSE_ERROR)
                .message("missing fields")
                .throw();
        }

        Ok(())
    }
}

pub fn collection_ids(collection_states: &Vec<(String, String, Arc<RwLock<CollectionState>>)>) -> Vec<String> {
    collection_states.iter()
        .map(|cs| cs.0.clone())
        .collect()
}
