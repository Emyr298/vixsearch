use actix_web::{HttpResponse, Responder, post, web};
use validator::Validate;

use crate::{
    errcode,
    orchestrator::{self},
    transport::http::request_response::{CreateCollectionRequest, http_error, http_map_error},
    utils,
};

pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_collection);
}

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
