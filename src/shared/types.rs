use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{errcode, vixerr::Error};

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    String,
    Int32,
    Vector,
}

impl FromStr for DataType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(DataType::String),
            "int32" => Ok(DataType::Int32),
            "vector" => Ok(DataType::Vector),
            _ => Err(Error::new(errcode::PARSE_ERROR, "Invalid data type")),
        }
    }
}
