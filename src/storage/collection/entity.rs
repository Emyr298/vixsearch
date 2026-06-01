use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{collection::{self, CreateStorageParam}, errcode, shared, utils::vixerr::Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub fields: Vec<Field>,
}

impl TryInto<collection::Collection> for Collection {
    type Error = Error;
    
    fn try_into(self) -> Result<collection::Collection, Self::Error> {
        let fields: Vec<collection::Field> = self.fields
            .into_iter()
            .map(|test| test.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(collection::Collection { id: self.id, name: self.name, fields: fields })
    }
}

impl From<CreateStorageParam> for Collection {
    fn from(param: CreateStorageParam) -> Self {
        Collection {
            id: param.id,
            name: param.name,
            fields: param.fields.into_iter().map(Field::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
}

impl TryInto<collection::Field> for Field {
    type Error = Error;
    
    fn try_into(self) -> Result<collection::Field, Self::Error> {
        let field_type = shared::ValueType::from_str(&self.field_type)?;
        Ok(collection::Field { name: self.name, field_type: field_type })
    }
}

impl From<collection::Field> for Field {
    fn from(field: collection::Field) -> Self {
        Field {
            name: field.name,
            field_type: field.field_type.into(),
        }
    }
}

pub fn parse_and_verify_content(content: &str) -> Result<&str, Error> {
    if content.len() < 9 || &content[8..9] != ":" {
        return Err(Error::new(errcode::FATAL_ERROR, "failed to parse and verify metadata"));
    }

    let crc_str = &content[..8];
    let json_str = &content[9..];

    let expected_crc = u32::from_str_radix(crc_str, 16).map_err(|_| Error::new(errcode::FATAL_ERROR, "failed to parse crc32"))?;
    let actual_crc = crc32fast::hash(&json_str.as_bytes());
    if expected_crc != actual_crc {
        return Err(Error::new(errcode::FATAL_ERROR, "invalid actual crc")); 
    }

    Ok(json_str)
}
