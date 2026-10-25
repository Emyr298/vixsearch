use actix_web::{
    HttpResponse, Result,
    http::{self, StatusCode},
};
use log::error;
use serde::Serialize;

use crate::{
    errcode,
    utils::{
        http::{HttpError, response_success},
        vixerr::Error,
    },
};

pub fn http_success<T: Serialize>(
    status_code: StatusCode,
    code: impl Into<String>,
    data: T,
) -> Result<HttpResponse> {
    Ok(HttpResponse::build(status_code).json(response_success(&code.into(), data)))
}

pub fn http_error(err: Error, message: impl Into<String>) -> Result<HttpResponse> {
    let message = message.into();
    error!("[{}] {}", err.code, err);
    HttpError::new(get_http_status_code(&err.code), &err.code, &message).throw()
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
