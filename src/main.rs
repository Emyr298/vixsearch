// use std::sync::Arc;

use actix_web::{App, HttpServer, web};
// use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;

mod collection;
mod config;
mod di;
mod errcode;
// mod shared;
mod storage;
mod transport;
mod translog;
mod utils;
mod document;
mod query;

use utils::vixerr;

use crate::{di::Application, utils::vixerr::Error};

fn main() {
    if dotenv().is_err() {
        println!(".env file is not found");
    }

    let app = di::register_dependencies();
    if let Err(err) = load(&app) {
        panic!("PANIC: {}", &err.message);
    }

    // let actix_thread = std::thread::spawn(move || start_actix(app.config, app.orchestrator));
    // actix_thread.join().unwrap();
}

fn load(app: &Application) -> Result<(), Error> {
    app.collection_loader.load()
}

// fn start_actix(config: config::Config) {
//     let orchestrator_data = web::Data::from(orchestrator);
//     actix_web::rt::System::new().block_on(async move {
//         HttpServer::new(move || {
//             App::new()
//                 .app_data(utils::http::json_error_handler(
//                     errcode::PARSE_ERROR,
//                     "failed to parse request",
//                 ))
//                 .app_data(orchestrator_data.clone())
//                 .configure(transport::http::register_routes)
//         })
//         .bind(config.address)
//         .unwrap()
//         .run()
//         .await
//         .unwrap();
//     });
// }
