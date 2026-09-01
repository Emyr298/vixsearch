use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{errcode::PARSE_ERROR, utils::vixerr::Error};

pub fn key_from_id(id: &str) -> Vec<u8> {
    format!("id_{}", id).into_bytes()
}

pub fn key_from_seq_id(seq_id: &u64) -> Vec<u8> {
    [b"seq_", &seq_id.to_le_bytes()[..]].concat()
}

#[derive(Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub seq_id: u64,
    pub payload: HashMap<String, Value>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum ValueType {
    String,
    Int64,
}

impl ValueType {
    pub fn new(value_type_str: &str) -> Result<Self, Error> {
        match value_type_str {
            "string" => Ok(ValueType::String),
            "int64" => Ok(ValueType::Int64),
            _ => Error::code(PARSE_ERROR)
                    .message(format!("invalid value type: {}", value_type_str))
                    .throw(),
        }
    }

    pub fn string(&self) -> String {
        match self {
            ValueType::String => "string".to_string(),
            ValueType::Int64 => "int64".to_string(),
        }
    }

    pub fn value(&self, raw_value: &RawValue) -> Result<Value, Error> {
        match self {
            ValueType::String => {
                match raw_value {
                    RawValue::String(s) => Ok(Value::String(s.to_string())),
                    default => return Error::code(PARSE_ERROR)
                        .message(format!("invalid value for {}: {}", self.string(), default))
                        .throw(),
                }
            }
            ValueType::Int64 => {
                match raw_value {
                    RawValue::Int64(n) => Ok(Value::Int64(n.clone())),
                    default => return Error::code(PARSE_ERROR)
                        .message(format!("invalid value for {}: {}", self.string(), default))
                        .throw(),
                }
            },
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int64(i64),
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::String(_) => ValueType::String,
            Value::Int64(_) => ValueType::Int64,
        }
    }
}

pub enum RawValue {
    String(String),
    Int64(i64),
    Float64(f64),
    Array(Vec<RawValue>),
}

impl fmt::Display for RawValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawValue::String(s) => write!(f, "{}", s),
            RawValue::Int64(n) => write!(f, "{}", n),
            RawValue::Float64(n) => write!(f, "{}", n),
            RawValue::Array(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}
