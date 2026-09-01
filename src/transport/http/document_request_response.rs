use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{document::{self}, errcode::PARSE_ERROR, query, utils::vixerr::Error};

#[derive(Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "_id")]
    pub id: String,

    #[serde(flatten)]
    pub payload: HashMap<String, serde_json::Value>,
}

impl Document {
    pub fn from_internal_document(doc: document::Document) -> Self {
        Self {
            id: doc.id,
            payload: doc.payload.into_iter()
                .map(|(k, v)| (k, serde_document_value(v)))
                .collect(),
        }
    }

    pub fn insert_param(self, collection_id: &str) -> Result<query::InsertParam, Error> {
        Ok(query::InsertParam {
            collection_id: collection_id.to_string(),
            document_id: self.id,
            payload: self.payload.into_iter()
                .map(|(k, v)| internal_document_value(v).map(|rv| (k, rv)))
                .collect::<Result<_, _>>()?,
        })
    }
}

fn internal_document_value(value: serde_json::Value) -> Result<document::RawValue, Error> {
    match value {
        serde_json::Value::String(s) => Ok(document::RawValue::String(s)),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Ok(document::RawValue::Int64(n.as_i64().unwrap()))
            } else if n.is_f64() {
                Ok(document::RawValue::Float64(n.as_f64().unwrap()))
            } else {
                return Error::code(PARSE_ERROR)
                    .message("unsupported data type")
                    .throw();
            }
        },
        _ => return Error::code(PARSE_ERROR)
            .message("unsupported data type")
            .throw(),
    }
}

fn serde_document_value(value: document::Value) -> serde_json::Value {
    match value {
        document::Value::String(s) => serde_json::Value::String(s),
        document::Value::Int64(n) => serde_json::Value::Number(n.into()),
    }
}
