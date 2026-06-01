use config::{Environment, File, FileFormat};
use serde::{Deserialize};

use crate::defaults::ConfigDefaults;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: String,
    pub data_dir: String,
    pub collection_metadata_filename: String,
}

impl Config {
    pub fn new() -> Self {
        let defaults = serde_json::to_value(ConfigDefaults::default()).expect("failed to serialize defaults");

        config::Config::builder()
            .add_source(File::from_str(&defaults.to_string(), FileFormat::Json))
            .add_source(Environment::default())
            .build()
            .expect("Failed to build config")
            .try_deserialize::<Config>()
            .expect("Failed to deserialize config")
    }
}
