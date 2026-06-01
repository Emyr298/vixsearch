use std::fmt::Display;

use actix_web::{HttpResponse, ResponseError, http, web};

use crate::utils::http::response_error;

#[derive(Debug)]
pub struct HttpError {
    code: String,
    status_code: http::StatusCode,
    message: String,
}

impl HttpError {
    pub fn new(status_code: http::StatusCode, code: String, message: String) -> Self {
        Self {
            code,
            status_code,
            message,
        }
    }
}

impl ResponseError for HttpError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.status_code
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        HttpResponse::build(self.status_code)
            .json(response_error(self.code.clone(), self.message.clone()))
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub fn error_handler(
    default_code: impl Into<String>,
    default_message: impl Into<String>,
) -> web::JsonConfig {
    let code = default_code.into();
    let message = default_message.into();

    web::JsonConfig::default().error_handler(move |_, _| {
        let err = HttpError::new(
            http::StatusCode::BAD_REQUEST,
            code.to_string(),
            message.to_string(),
        );
        let response = err.error_response();
        actix_web::error::InternalError::from_response(err, response).into()
    })
}
