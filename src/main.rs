// use std::sync::Arc;

use actix_web::{App, HttpServer, web};
// use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;

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
    utils::{
        http::{json_error_handler, not_found_route},
        vixerr::Error,
    },
};

fn main() {
    if dotenv().is_err() {
        println!(".env file is not found");
    }

    env_logger::init();

    let app = di::register_dependencies();
    if let Err(err) = load(&app) {
        panic!("PANIC: {}", &err.message);
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
                .app_data(json_error_handler(
                    errcode::PARSE_ERROR,
                    "failed to parse request",
                ))
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
