use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{errcode, vixerr::Error};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize)]
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
