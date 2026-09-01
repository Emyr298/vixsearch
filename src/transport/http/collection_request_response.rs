use std::collections::HashMap;

use actix_web::{Error, HttpResponse, Result, http::{self, StatusCode}};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    collection::{CreateParam, CreateParamField}, document::ValueType, errcode, utils::{self, http::{HttpError, response_error, response_success}, vixerr},
};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Int32,
}

impl DataType {
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
            fields: self.fields.iter()
                .map(|f| CreateParamField {
                    name: f.name.clone(),
                    field_type: f.field_type.value_type(),
                })
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CreateCollectionRequestField {
    #[validate(length(min = 1, max = 32))]
    pub name: String,

    pub field_type: DataType,
}
