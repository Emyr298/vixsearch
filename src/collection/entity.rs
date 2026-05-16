use crate::shared::DataType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: DataType,
}
