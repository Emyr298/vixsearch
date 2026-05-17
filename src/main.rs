use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;

mod collection;
mod config;
mod errcode;
mod orchestrator;
mod query;
mod shared;
mod translog;
mod transport;
mod utils;

use utils::vixerr;

fn main() {
    if dotenv().is_err() {
        println!(".env file is not found");
    }

    let config = config::Config::new();
    let collection_manager = collection::new_manager(&config.data_dir);
    let translog_manager = translog::new_manager();
    let orchestrator: Arc<dyn orchestrator::Orchestrator> =
        orchestrator::new_orchestrator(collection_manager, translog_manager).into();

    let actix_thread = std::thread::spawn(move || run_actix(config, orchestrator));
    actix_thread.join().unwrap();
}

fn run_actix(config: config::Config, orchestrator: Arc<dyn orchestrator::Orchestrator>) {
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
