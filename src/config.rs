use std::sync::Arc;

use config::{Environment, File, FileFormat};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: String,
    pub base_dir: String,
    pub base_temp_dir: String,
}

impl Config {
    pub fn new() -> Arc<Self> {
        let defaults =
            serde_json::to_value(ConfigDefaults::default()).expect("failed to serialize defaults");

        let cfg = config::Config::builder()
            .add_source(File::from_str(&defaults.to_string(), FileFormat::Json))
            .add_source(Environment::default())
            .build()
            .expect("Failed to build config")
            .try_deserialize::<Config>()
            .expect("Failed to deserialize config");

        Arc::new(cfg)
    }
}

#[derive(Serialize)]
struct ConfigDefaults {
    address: &'static str,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8080",
        }
    }
}
