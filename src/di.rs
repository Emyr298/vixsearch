use std::sync::Arc;

use crate::{
    collection::{self, CollectionAdapter, CollectionLifecycleEvent, CollectionLoader},
    config::Config,
    storage::FileStorage,
    utils::handler::handler_group::HandlerGroup,
};

pub struct Application {
    pub collection_service: Arc<dyn collection::CollectionService>,
    pub collection_loader: Arc<dyn CollectionLoader>,
    pub config: Arc<Config>,
}

impl Application {
    fn new(
        collection_service: Arc<dyn collection::CollectionService>,
        collection_loader: Arc<dyn CollectionLoader>,
        config: Arc<Config>,
    ) -> Self {
        Application {
            collection_service,
            collection_loader,
            config,
        }
    }
}

pub fn register_dependencies() -> Application {
    // Infrastructure
    let config = Config::new();
    let file_storage = FileStorage::new(&config.base_dir, &config.base_temp_dir);
    let (collection_lifecycle_handlers, _) =
        HandlerGroup::<CollectionLifecycleEvent>::new(Vec::new());

    // Adapter
    let collection_adapter = CollectionAdapter::new(file_storage);

    // Service/Engine
    let (collection_service, collection_loader) =
        collection::ServiceImpl::new(collection_adapter, collection_lifecycle_handlers);

    return Application::new(collection_service, collection_loader, config);
}
