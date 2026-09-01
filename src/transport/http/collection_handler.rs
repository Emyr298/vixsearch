



use std::sync::Arc;

use actix_web::{HttpResponse, Result, delete, http::StatusCode, post, web::{Data, Json, Path, ServiceConfig, block}};
use validator::Validate;

use crate::{collection::CollectionService, errcode::PARSE_ERROR, transport::http::{collection_request_response::CreateCollectionRequest, request_response::{http_error, http_success}}, utils::http::response_success};

pub fn register_collection_routes(cfg: &mut ServiceConfig) {
    cfg.service(create_collection);
    cfg.service(delete_collection);
}

#[post("/collections")]
async fn create_collection(
    payload: Json<CreateCollectionRequest>,
    collection_service: Data<Arc<dyn CollectionService>>,
) -> Result<HttpResponse> {
    if let Err(e) = payload.validate() {
        return http_error(PARSE_ERROR, "failed to validate request");
    }

    let result = block(move || collection_service.create(payload.create_param())).await.unwrap();
    if let Err(e) = result {
        return http_error(e.code, e.message);
    }

    http_success(StatusCode::CREATED, "SUCCESS", ())
}

#[delete("/collections/{collection}")]
async fn delete_collection(
    collection: Path<String>,
    collection_service: Data<Arc<dyn CollectionService>>,
) -> Result<HttpResponse> {
    let collection_id = collection.into_inner();

    let result = block(move || collection_service.delete_by_id(&collection_id)).await.unwrap();
    if let Err(e) = result {
        return http_error(e.code, e.message);
    }

    http_success(StatusCode::ACCEPTED, "SUCCESS", ())
}
