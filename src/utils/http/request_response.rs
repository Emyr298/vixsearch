use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: String,
    pub data: T,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

#[derive(Serialize)]
pub struct ApiResponseError {
    pub code: String,
    pub errors: Vec<String>,
}

pub fn response_success<T: Serialize>(code: String, data: T) -> ApiResponse<T> {
    ApiResponse {
        code: code,
        data,
        errors: Vec::new(),
    }
}

pub fn response_error(code: String, error: String) -> ApiResponseError {
    ApiResponseError {
        code,
        errors: vec![error],
    }
}

pub fn response_errors(code: String, errors: Vec<String>) -> ApiResponseError {
    ApiResponseError { code, errors }
}
