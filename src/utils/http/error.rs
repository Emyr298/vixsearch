use std::fmt;

use actix_web::{Error, HttpResponse, Responder, ResponseError, body::BoxBody, http::StatusCode};

use crate::utils::http::response_error;

#[derive(Debug)]
pub struct HttpError {
    code: String,
    status_code: StatusCode,
    message: String,
}

impl HttpError {
    pub fn new(status_code: StatusCode, code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            status_code,
            message: message.to_string(),
        }
    }

    pub fn throw(self) -> Result<HttpResponse, Error> {
        Err::<HttpResponse, Error>(self.into())
    }
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code)
            .json(response_error(&self.code, &self.message))
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.status_code, self.code, self.message)
    }
}
