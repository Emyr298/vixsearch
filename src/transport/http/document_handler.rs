



use std::{collections::HashMap, sync::Arc};

use actix_web::{HttpResponse, Result, delete, http::StatusCode, post, web::{Data, Json, Path, ServiceConfig, block}};
use serde_json::Value;
use validator::Validate;

use crate::{collection::CollectionService, errcode::PARSE_ERROR, query::QueryService, transport::http::{document_request_response::Document, request_response::{http_error, http_success}}, utils::http::response_success};

// TODO: handle FATAL_ERROR as middleware
pub fn register_document_routes(cfg: &mut ServiceConfig) {
    cfg.service(insert_document);
}

#[post("/collections/{collection}/documents")]
async fn insert_document(
    collection_id_path: Path<String>,
    document_request_json: Json<Document>,
    query_service: Data<Arc<dyn QueryService>>,
) -> Result<HttpResponse> {
    let collection_id = collection_id_path.into_inner();
    let document_request = document_request_json.into_inner();

    let param = match document_request.insert_param(&collection_id) {
        Ok(p) => p,
        Err(e) => return http_error(e.code, e.message),
    };

    let document_response = match query_service.insert(param) {
        Ok(d) => d,
        Err(e) => return http_error(e.code, e.message),
    };
    
    http_success(StatusCode::OK, "SUCCESS", Document::from_internal_document(document_response))
}
