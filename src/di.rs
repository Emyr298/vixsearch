use std::sync::Arc;

use crate::{collection, config, orchestrator, storage};

pub struct Application {
    pub orchestrator: Arc<dyn orchestrator::Orchestrator>,
    pub config: config::Config,
}

impl Application {
    pub fn new(orchestrator: Arc<dyn orchestrator::Orchestrator>, config: config::Config) -> Self {
        Application {
            orchestrator,
            config,
        }
    }
}

pub fn register_dependencies() -> Application {
    let config = config::Config::new();
    let collection_storage =
        storage::new_storage(&config.data_dir, &config.collection_metadata_filename);
    let collection_manager = collection::new_manager(collection_storage);
    let orchestrator: Arc<dyn orchestrator::Orchestrator> =
        orchestrator::new_orchestrator(collection_manager).into();

    return Application::new(orchestrator, config);
}
