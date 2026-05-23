use std::collections::HashMap;

use actix_web::{HttpResponse, Responder, post, web};
use validator::Validate;

use crate::{
    errcode,
    orchestrator::{self},
    transport::http::request_response::{
        CreateCollectionRequest, http_error, http_map_error, new_document,
    },
    utils,
};

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_collection);
    cfg.service(insert_document);
}

// TODO: new thread to make async fn not blocking

#[post("/collections")]
async fn create_collection(
    payload: web::Json<CreateCollectionRequest>,
    orchestrator: web::Data<dyn orchestrator::Orchestrator>,
) -> actix_web::Result<impl Responder> {
    payload
        .validate()
        .map_err(|_| http_error(errcode::PARSE_ERROR, "failed to validate request"))?;

    let param = orchestrator::CreateCollectionParam::from(payload.into_inner());

    orchestrator
        .create_collection(param)
        .map_err(|e| http_map_error(e))?;

    Ok(HttpResponse::Created().json(utils::http::response_success("SUCCESS".to_string(), ())))
}

#[post("/collections/{collection}/documents")]
async fn insert_document(
    collection: web::Path<String>,
    payload: web::Json<HashMap<String, serde_json::Value>>,
    orchestrator: web::Data<dyn orchestrator::Orchestrator>,
) -> actix_web::Result<impl Responder> {
    let collection_name = collection.into_inner();
    let document = new_document(payload.into_inner()).map_err(|e| http_map_error(e))?;

    orchestrator
        .insert_document(collection_name, document)
        .map_err(|e| http_map_error(e))?;

    Ok(HttpResponse::Created().json(utils::http::response_success("SUCCESS".to_string(), ())))
}
