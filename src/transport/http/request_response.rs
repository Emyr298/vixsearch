use std::collections::HashMap;

use actix_web::{Error, HttpResponse, Result, http::{self, StatusCode}};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    collection::{CreateParam, CreateParamField}, document::ValueType, errcode, utils::{self, http::{HttpError, response_error, response_success}, vixerr},
};

pub fn http_success<T: Serialize>(status_code: StatusCode, code: impl Into<String>, data: T) -> Result<HttpResponse> {
    Ok(HttpResponse::build(status_code)
        .json(response_success(&code.into(), data)))
}

pub fn http_error(code: impl Into<String>, message: impl Into<String>) -> Result<HttpResponse> {
    let code = code.into();
    let message = message.into();
    HttpError::new(get_http_status_code(&code), &code, &message).throw()
}

fn get_http_status_code(code: &str) -> http::StatusCode {
    match code {
        errcode::SYSTEM_ERROR => http::StatusCode::INTERNAL_SERVER_ERROR,
        errcode::PARSE_ERROR => http::StatusCode::BAD_REQUEST,
        errcode::NOT_FOUND => http::StatusCode::NOT_FOUND,
        errcode::EXISTS => http::StatusCode::CONFLICT,
        _ => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}
