use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;

mod collection;
mod defaults;
mod config;
mod di;
mod errcode;
mod orchestrator;
mod shared;
mod storage;
mod transport;
mod utils;
mod index;
mod document;
mod query;

use utils::vixerr;

use crate::{di::Application, utils::vixerr::Error};

fn main() {
    if dotenv().is_err() {
        println!(".env file is not found");
    }

    let app = di::register_dependencies();
    if let Err(err) = load_data(&app) {
        panic!("PANIC: {}", &err.message);
    }

    let actix_thread = std::thread::spawn(move || start_actix(app.config, app.orchestrator));
    actix_thread.join().unwrap();
}

fn load_data(app: &Application) -> Result<(), Error> {
    app.orchestrator.load_collection()?;

    let temp_collection_ids: Vec<String> = Vec::new();
    app.document_loader.load(&temp_collection_ids)?;

    Ok(())
}

fn start_actix(config: config::Config, orchestrator: Arc<dyn orchestrator::Orchestrator>) {
    let orchestrator_data = web::Data::from(orchestrator);
    actix_web::rt::System::new().block_on(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(utils::http::error_handler(
                    errcode::PARSE_ERROR,
                    "failed to parse request",
                ))
                .app_data(orchestrator_data.clone())
                .configure(transport::http::register_routes)
        })
        .bind(config.address)
        .unwrap()
        .run()
        .await
        .unwrap();
    });
}
