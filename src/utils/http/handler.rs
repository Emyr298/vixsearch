use actix_web::{HttpResponse, Responder, http::StatusCode, web::JsonConfig};
use serde_json::json;

use crate::utils::http::HttpError;

pub fn json_error_handler(
    default_code: impl Into<String>,
    default_message: impl Into<String>,
) -> JsonConfig {
    let code = default_code.into();
    let message = default_message.into();

    JsonConfig::default().error_handler(move |_, _| {
        let err = HttpError::new(StatusCode::BAD_REQUEST, code.as_str(), message.as_str());
        err.into()
    })
}

pub async fn not_found_route() -> HttpResponse {
    HttpResponse::NotFound().json(json!({
        "code": "NOT_FOUND",
        "message": "route not found",
    }))
}
