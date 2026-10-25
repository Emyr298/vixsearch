use actix_web::{
    HttpResponse, Result, delete, get,
    http::StatusCode,
    post,
    web::{Data, Json, Path, ServiceConfig, block},
};
use validator::Validate;

use crate::{
    collection::CollectionService,
    errcode::PARSE_ERROR,
    transport::http::{
        collection_request_response::{CreateCollectionRequest, GetByIDResponse},
        request_response::{http_error, http_success},
    },
    utils::vixerr::Error,
};

pub fn register_collection_routes(cfg: &mut ServiceConfig) {
    cfg.service(get_collection);
    cfg.service(create_collection);
    cfg.service(delete_collection);
}

#[get("/collection/{collection}")]
async fn get_collection(
    collection_id: Path<String>,
    collection_service: Data<dyn CollectionService>,
) -> Result<HttpResponse> {
    let collection_id = collection_id.into_inner();

    let result = block(move || collection_service.get_by_id(&collection_id))
        .await
        .unwrap();

    let collection = match result {
        Ok(c) => c,
        Err(e) => return http_error(e, "failed to get collection"),
    };

    http_success(
        StatusCode::OK,
        "SUCCESS",
        GetByIDResponse::from_result(&collection),
    )
}

#[post("/collection")]
async fn create_collection(
    payload: Json<CreateCollectionRequest>,
    collection_service: Data<dyn CollectionService>,
) -> Result<HttpResponse> {
    if let Err(_) = payload.validate() {
        return http_error(Error::code(PARSE_ERROR), "failed to validate request");
    }

    let result = block(move || collection_service.create(payload.create_param()))
        .await
        .unwrap();
    if let Err(e) = result {
        return http_error(e, "failed to create collection");
    }

    http_success(StatusCode::CREATED, "SUCCESS", ())
}

#[delete("/collection/{collection}")]
async fn delete_collection(
    collection: Path<String>,
    collection_service: Data<dyn CollectionService>,
) -> Result<HttpResponse> {
    let collection_id = collection.into_inner();

    let result = block(move || collection_service.delete_by_id(&collection_id))
        .await
        .unwrap();
    if let Err(e) = result {
        return http_error(e, "failed to delete collection");
    }

    http_success(StatusCode::ACCEPTED, "SUCCESS", ())
}
