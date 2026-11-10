use actix_web::{
    HttpResponse, Result,
    http::{self, StatusCode},
};
use log::error;
use serde::Serialize;

use crate::{
    errcode,
    utils::{
        http::{response_error, response_success},
        vixerr,
    },
};

pub fn http_success<T: Serialize>(
    status_code: StatusCode,
    code: impl Into<String>,
    data: T,
) -> Result<HttpResponse> {
    Ok(response_success(status_code, &code.into(), data))
}

pub fn http_error(e: impl std::error::Error + 'static) -> Result<HttpResponse> {
    let mut code = errcode::SYSTEM_ERROR.to_string();
    if let Some(ve) = (&e as &dyn std::error::Error).downcast_ref::<vixerr::Error>() {
        code = ve.code.clone();
    }

    error!("[{}] {}", code, e);
    if code == errcode::FATAL_ERROR {
        std::process::abort();
    }

    let status_code = get_http_status_code(&code);

    let mut message = None;
    if let Some(ve) = (&e as &dyn std::error::Error).downcast_ref::<vixerr::Error>() {
        message = ve.message.clone();
    }

    Ok(response_error(status_code, &code, message))
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
