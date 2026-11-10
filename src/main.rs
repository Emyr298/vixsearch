use actix_web::{
    App, HttpResponse, HttpServer,
    error::InternalError,
    http::StatusCode,
    web::{self, JsonConfig},
};
use dotenvy::dotenv;
use log::error;

mod collection;
mod common;
mod config;
mod di;
mod errcode;
mod storage;
mod transport;
mod utils;

use utils::vixerr;

use crate::{
    di::Application,
    transport::http::register_collection_routes,
    utils::{http::response_error, vixerr::Error},
};

fn main() {
    if dotenv().is_err() {
        println!(".env file is not found");
    }

    env_logger::init();

    let app = di::register_dependencies();
    if let Err(err) = load(&app) {
        error!("[{}] {}", err.code, err);
        std::process::abort();
    }

    let actix_thread = std::thread::spawn(move || start_actix(&app));
    actix_thread.join().unwrap();
}

fn load(app: &Application) -> Result<(), Error> {
    app.collection_loader.load()
}

fn start_actix(app: &Application) {
    let orchestrator_data = web::Data::from(app.collection_service.clone());
    actix_web::rt::System::new().block_on(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(json_error_handler())
                .app_data(orchestrator_data.clone())
                .configure(register_collection_routes)
                .default_service(web::route().to(not_found_route))
        })
        .bind(&app.config.address)
        .unwrap()
        .run()
        .await
        .unwrap();
    });
}

pub fn json_error_handler() -> JsonConfig {
    JsonConfig::default().error_handler(move |e, _| {
        let response = response_error(
            StatusCode::BAD_REQUEST,
            errcode::PARSE_ERROR,
            Some(e.to_string()),
        );
        InternalError::from_response(e, response).into()
    })
}

pub async fn not_found_route() -> HttpResponse {
    response_error(
        StatusCode::NOT_FOUND,
        errcode::NOT_FOUND,
        Some("route not found".to_string()),
    )
}
