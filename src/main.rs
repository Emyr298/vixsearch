use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;

mod collection;
mod config;
mod di;
mod errcode;
mod orchestrator;
mod shared;
mod storage;
mod transport;
mod utils;

use utils::vixerr;

fn main() {
    if dotenv().is_err() {
        println!(".env file is not found");
    }

    let app = di::register_dependencies();
    load_data(&app.orchestrator);

    let actix_thread = std::thread::spawn(move || start_actix(app.config, app.orchestrator));
    actix_thread.join().unwrap();
}

fn load_data(orchestrator: &Arc<dyn orchestrator::Orchestrator>) {
    orchestrator.load_collection().expect("PANIC: failed to load")
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
