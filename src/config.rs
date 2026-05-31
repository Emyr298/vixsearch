use config::Environment;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub address: String,
    pub data_dir: String,
    pub collection_metadata_filename: String,
}

impl Config {
    pub fn new() -> Self {
        config::Config::builder()
            .add_source(Environment::default())
            .build()
            .expect("Failed to build config")
            .try_deserialize::<Config>()
            .expect("Failed to deserialize config")
    }
}
