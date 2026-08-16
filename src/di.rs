use std::sync::Arc;

use crate::{collection, config, document::{self, LSMAdapter, LSMEngine, Loader}, orchestrator, storage::{self, file_storage::FileStorage}, utils::vixpool::StandardPool};

pub struct Application {
    pub orchestrator: Arc<dyn orchestrator::Orchestrator>,
    pub document_loader: Arc<dyn Loader>,
    pub config: config::Config,
}

impl Application {
    pub fn new(
        orchestrator: Arc<dyn orchestrator::Orchestrator>,
        document_loader: Arc<dyn Loader>,
        config: config::Config,
    ) -> Self {
        Application {
            orchestrator,
            document_loader,
            config,
        }
    }
}

pub fn register_dependencies() -> Application {
    // Infrastructure
    let config = config::Config::new();
    let storage = FileStorage::new(&config.base_dir);
    let flush_pool = StandardPool::new(
        config.document_flush_thread_size,
        Some(config.document_flush_queue_size),
        true,
    );

    // Adapter
    let lsm_adapter = LSMAdapter::new(
        storage,
        config.document_block_min_content_size,
        config.document_bloomfilter_false_positive_probability,
    );

    // Manager/Engine
    let (document_engine, document_loader) = LSMEngine::new(
        lsm_adapter,
        flush_pool,
        config.document_flush_byte_size_threshold,
    );
    let document_manager = document::ManagerImpl::new(document_engine);
    
    // Old
    let collection_storage =
        storage::new_storage(&config.base_dir, &config.collection_metadata_filename);
    let collection_manager = collection::new_manager(collection_storage);

    let orchestrator: Arc<dyn orchestrator::Orchestrator> =
        orchestrator::new_orchestrator(collection_manager).into();

    return Application::new(
        orchestrator,
        document_loader,
         config,
    );
}
