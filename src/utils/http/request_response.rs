use serde::Serialize;

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

pub fn response_success<T: Serialize>(code: &str, data: T) -> ApiResponse<T> {
    ApiResponse {
        code: code.to_string(),
        data,
    }
}

pub fn response_error(code: &str, error: &str) -> ApiResponseError {
    ApiResponseError {
        code: code.to_string(),
        errors: vec![error.to_string()],
    }
}

pub fn response_errors(code: &str, errors: &[String]) -> ApiResponseError {
    ApiResponseError {
        code: code.to_string(),
        errors: errors.to_vec(),
    }
}
