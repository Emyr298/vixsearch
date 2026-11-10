use actix_web::{HttpResponse, http::StatusCode};
use serde::Serialize;

const DEFAULT_ERROR_MESSAGE: &str = "An error occurred";

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: String,
    pub data: T,
}

#[derive(Serialize)]
pub struct ApiResponseError {
    pub code: String,
    pub errors: Vec<String>,
}

pub fn response_success<T: Serialize>(
    status_code: StatusCode,
    code: &str,
    data: T,
) -> HttpResponse {
    HttpResponse::build(status_code).json(ApiResponse {
        code: code.to_string(),
        data,
    })
}

pub fn response_error(
    status_code: StatusCode,
    code: &str,
    message: Option<String>,
) -> HttpResponse {
    HttpResponse::build(status_code).json(ApiResponseError {
        code: code.to_string(),
        errors: vec![message.unwrap_or(DEFAULT_ERROR_MESSAGE.to_string())],
    })
}
