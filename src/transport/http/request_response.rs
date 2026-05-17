use actix_web::http;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    errcode, orchestrator, shared,
    utils::{self, vixerr},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Int32,
    Vector,
}

impl From<DataType> for shared::DataType {
    fn from(val: DataType) -> Self {
        match val {
            DataType::String => Self::String,
            DataType::Int32 => Self::Int32,
            DataType::Vector => Self::Vector,
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

impl From<CreateCollectionRequest> for orchestrator::CreateCollectionParam {
    fn from(value: CreateCollectionRequest) -> Self {
        orchestrator::CreateCollectionParam {
            name: value.name,
            fields: value.fields.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CreateCollectionRequestField {
    #[validate(length(min = 1, max = 32))]
    pub name: String,

    pub field_type: DataType,
}

impl From<CreateCollectionRequestField> for orchestrator::CreateCollectionParamField {
    fn from(value: CreateCollectionRequestField) -> Self {
        orchestrator::CreateCollectionParamField {
            name: value.name,
            field_type: shared::DataType::from(value.field_type),
        }
    }
}

pub fn http_error(code: impl Into<String>, message: impl Into<String>) -> utils::http::HttpError {
    let code = code.into();
    let message = message.into();
    utils::http::HttpError::new(get_http_status_code(&code), code, message)
}

pub fn http_map_error(e: vixerr::Error) -> utils::http::HttpError {
    http_error(e.code, e.message)
}

fn get_http_status_code(code: &str) -> http::StatusCode {
    match code {
        errcode::SYSTEM_ERROR => http::StatusCode::INTERNAL_SERVER_ERROR,
        errcode::PARSE_ERROR => http::StatusCode::BAD_REQUEST,
        _ => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}
