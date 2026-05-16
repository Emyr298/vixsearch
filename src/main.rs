use dotenvy::dotenv;

use crate::collection::{CreateCollectionParam, Field, FieldType};

mod collection;
mod config;
mod error;
mod orchestrator;
mod query;
mod shared;
mod translog;
mod transport;

fn main() {
    if dotenv().is_err() {
        println!(".env file is not found");
    }

    let collection_manager = collection::new_manager(&std::env::var(config::DATA_DIR).unwrap());
    collection_manager
        .create_collection(CreateCollectionParam {
            name: "blogs".to_string(),
            fields: vec![Field {
                name: "title".to_string(),
                field_type: FieldType::String(String::from("Title")),
            }],
        })
        .unwrap();
}
