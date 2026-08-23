use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{errcode::PARSE_ERROR, utils::vixerr::Error};

// pub const IDENTIFIER_FIELD: &str = "_id";

pub fn key_from_id(id: &str) -> Vec<u8> {
    format!("id_{}", id).into_bytes()
}

pub fn key_from_seq_id(seq_id: &u64) -> Vec<u8> {
    [b"seq_", &seq_id.to_le_bytes()[..]].concat()
}

#[derive(Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub seq_id: String,
    pub payload: HashMap<String, Value>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ValueType {
    String,
    Int32,
    Vector,
}

impl ValueType {
    pub fn new(value_type_str: &str) -> Result<Self, Error> {
        match value_type_str {
            "string" => Ok(ValueType::String),
            "int32" => Ok(ValueType::Int32),
            "vector" => Ok(ValueType::Vector),
            _ => Error::code(PARSE_ERROR)
                    .message(format!("Invalid value type: {}", value_type_str))
                    .throw(),
        }
    }

    pub fn string(&self) -> String {
        match self {
            ValueType::String => "string".to_string(),
            ValueType::Int32 => "int32".to_string(),
            ValueType::Vector => "vector".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Value {
    String(String),
    Integer(i64),
    Vector(Vec<f64>),
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::String(_) => ValueType::String,
            Value::Integer(_) => ValueType::Int32,
            Value::Vector(_) => ValueType::Vector,
        }
    }
}

