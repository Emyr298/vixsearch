use std::sync::Arc;

use crate::{collection::{self, CollectionAdapter, CollectionLoader}, config::Config, document::{self, LSMDocumentAdapter, LSMDocumentEngine}, storage::{LegacyFileStorage, FileStorage}, utils::{observer::observer_group::ObserverGroup, vixpool::StandardPool}};

pub struct Application {
    pub collection_loader: Arc<dyn CollectionLoader>,
    pub _config: Config,
}

impl Application {
    fn new(
        collection_loader: Arc<dyn CollectionLoader>,
        _config: Config,
    ) -> Self {
        Application {
            collection_loader,
            _config,
        }
    }
}

pub fn register_dependencies() -> Application {
    // Infrastructure
    let config = Config::new();
    let storage = LegacyFileStorage::new(&config.base_dir);
    let worm_storage = FileStorage::new(&config.base_dir, &config.base_temp_dir);
    let flush_pool = StandardPool::new(
        config.document_flush_thread_size,
        Some(config.document_flush_queue_size),
        true,
    );

    // Adapter
    let lsm_adapter = LSMDocumentAdapter::new(
        storage.clone(),
        config.document_block_min_content_size,
        config.document_bloomfilter_false_positive_probability,
    );
    let collection_adapter = CollectionAdapter::new(worm_storage);

    // Service/Engine
    let (document_engine, document_engine_collection_lifecycle) = LSMDocumentEngine::new(
        lsm_adapter,
        flush_pool,
        config.document_flush_byte_size_threshold,
    );
    let (_document_service, document_collection_lifecycle) = document::DocumentServiceImpl::new(document_engine, document_engine_collection_lifecycle.clone());
    let (_collection_service, collection_loader) = collection::ServiceImpl::new(collection_adapter, document_collection_lifecycle);

    return Application::new(
        collection_loader,
        config,
    );
}
