use actix_web::{HttpResponse, Responder, post, web};
use validator::Validate;

use crate::{
    orchestrator::{self},
    transport::http::request_response::CreateCollectionRequest,
};

#[post("/collections")]
async fn create_collection(
    payload: web::Json<CreateCollectionRequest>,
    orchestrator: web::Data<dyn orchestrator::Orchestrator>,
) -> actix_web::Result<impl Responder> {
    payload
        .validate()
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    let param = orchestrator::CreateCollectionParam::try_from(payload.into_inner())
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    orchestrator
        .create_collection(param)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Created().finish())
}
