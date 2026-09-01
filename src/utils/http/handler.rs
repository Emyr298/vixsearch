use actix_web::{http::StatusCode, web::JsonConfig};

use crate::utils::http::HttpError;

// pub fn json_error_handler(
//     default_code: impl Into<String>,
//     default_message: impl Into<String>,
// ) -> JsonConfig {
//     let code = default_code.into();
//     let message = default_message.into();

//     JsonConfig::default().error_handler(move |_, _| {
//         let err = HttpError::new(
//             StatusCode::BAD_REQUEST,
//             code.to_string(),
//             message.to_string(),
//         );
//         err.actix_error()
//     })
// }
