use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{errcode, vixerr::Error};

pub const IDENTIFIER_FIELD: &str = "_id";

pub type Document = HashMap<String, Value>;

pub fn get_id(document: &Document) -> Result<String, Error> {
    let Some(id_value) = document.get(IDENTIFIER_FIELD) else {
        return Err(Error::new(errcode::FATAL_ERROR, "id not in document"));
    };

    let Value::String(id) = id_value else {
        return Err(Error::new(errcode::FATAL_ERROR, "id must be a string"));
    };

    Ok(id.clone())
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum ValueType {
    String,
    Int32,
    Vector,
}

impl FromStr for ValueType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(ValueType::String),
            "int32" => Ok(ValueType::Int32),
            "vector" => Ok(ValueType::Vector),
            _ => Err(Error::new(errcode::PARSE_ERROR, "Invalid value type")),
        }
    }
}

impl Into<String> for ValueType {
    fn into(self) -> String {
        match self {
            ValueType::String => "string".to_string(),
            ValueType::Int32 => "int32".to_string(),
            ValueType::Vector => "vector".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
