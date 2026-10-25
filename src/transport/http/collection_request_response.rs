use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    collection::{CreateParam, CreateParamField, GetByIDResult},
    common::ValueType,
};

// Request

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Int32,
}

impl DataType {
    fn from_value_type(value_type: &ValueType) -> Self {
        match value_type {
            ValueType::String => DataType::String,
            ValueType::Int64 => DataType::Int32,
        }
    }

    fn value_type(&self) -> ValueType {
        match self {
            DataType::String => ValueType::String,
            DataType::Int32 => ValueType::Int64,
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CreateCollectionRequest {
    #[validate(length(min = 1, max = 32))]
    pub name: String,

    #[validate(length(min = 1))]
    pub fields: Vec<CreateCollectionRequestField>,
}

impl CreateCollectionRequest {
    pub fn create_param(&self) -> CreateParam {
        CreateParam {
            id: self.name.clone(),
            fields: self
                .fields
                .iter()
                .map(|f| CreateParamField {
                    name: f.name.clone(),
                    field_type: f.field_type.value_type(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CreateCollectionRequestField {
    #[validate(length(min = 1, max = 32))]
    pub name: String,

    pub field_type: DataType,
}

// Response

#[derive(Serialize)]
pub struct GetByIDResponse {
    pub id: String,
    pub fields: Vec<GetByIDResponseField>,
}

impl GetByIDResponse {
    pub fn from_result(result: &GetByIDResult) -> Self {
        Self {
            id: result.id.clone(),
            fields: result
                .fields
                .iter()
                .map(|f| GetByIDResponseField {
                    name: f.name.clone(),
                    field_type: DataType::from_value_type(&f.field_type),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct GetByIDResponseField {
    pub name: String,
    pub field_type: DataType,
}
