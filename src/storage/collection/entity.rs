use serde::{Deserialize, Serialize};

use crate::collection::{self, CreateStorageParam};

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub fields: Vec<Field>,
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

impl From<collection::Field> for Field {
    fn from(field: collection::Field) -> Self {
        Field {
            name: field.name,
            field_type: field.field_type.into(),
        }
    }
}
