use serde::Serialize;

#[derive(Serialize)]
pub struct ConfigDefaults {
    address: &'static str,
    collection_metadata_filename: &'static str,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            address: "0.0.0.0:8080",
            collection_metadata_filename: "metadata.cjson",
        }
    }
}
