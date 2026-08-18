use std::sync::{Arc, RwLock};

use dashmap::{DashMap, Entry};

use crate::{collection::{service::{Loader, Port, Service}, service_param_result::CreateParam, state::{CollectionState, CollectionStatus}}, document::Document, errcode::{EXISTS, NOT_FOUND, NOT_READY}, utils::vixerr::Error};

pub struct ServiceImpl {
    adapter: Arc<dyn Port>,
    collection_by_id: DashMap<String, Arc<RwLock<CollectionState>>>
}

impl ServiceImpl {
    pub fn new(adapter: Arc<dyn Port>) -> Arc<dyn Service> {
        Arc::new(ServiceImpl {
            adapter,
            collection_by_id: DashMap::new(),
        })
    }
}

impl Loader for ServiceImpl {
    fn load(&self) -> Result<(), Error> {
        let result = self.adapter.get_all()?;

        let collection_states: Vec<(String, Arc<RwLock<CollectionState>>)> = result.collections.iter()
            .map(|c| CollectionState::from_get_all_port_result_collection(c, CollectionStatus::Loading))
            .map(|c| (c.id.to_string(), Arc::new(RwLock::new(c))))
            .collect();

        for (id, collection_state) in &collection_states {
            self.collection_by_id.insert(id.to_string(), collection_state.clone());
        }

        // TODO: sync

        for (_, collection_state) in &collection_states {
            let mut collection_write = collection_state.write().unwrap();
            collection_write.status = CollectionStatus::Ready;
        }

        Ok(())
    }
}

impl Service for ServiceImpl {
    fn create(&self, param: CreateParam) -> Result<(), Error> {
        param.validate()?;

        let collection_entry = self.collection_by_id.entry(param.id.to_string());
        if let Entry::Occupied(_) = &collection_entry {
            return Error::code(EXISTS)
                .message("collection already exists")
                .throw();
        }

        let collection_arc = Arc::new(RwLock::new(CollectionState::new(
            &param.id,
            &param.fields,
            CollectionStatus::Loading,
        )));

        collection_entry.insert(collection_arc.clone()); // early drop for dashmap shard, so it doesn't need to wait

        // TODO: sync to document service, etc.

        if let Err(e) = self.adapter.create(param.create_port_param()) {
            self.collection_by_id.remove(&param.id);
            return Err(e);
        }

        let mut collection = collection_arc.write().unwrap();
        collection.status = CollectionStatus::Ready;

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), Error> {
        let collection_ref = match self.collection_by_id.get(id) {
            Some(c) => c,
            None => return Error::code(NOT_FOUND)
                .message("collection not found")
                .throw(),
        };

        let collection_arc = collection_ref.value().clone();
        drop(collection_ref);

        let collection = collection_arc.write().unwrap();
        if collection.status == CollectionStatus::Deleting {
            return Ok(());
        }

        if let Err(e) = self.adapter.delete(&collection.id) {
            return Err(e);
        }

        // TODO: sync to document service, etc.

        self.collection_by_id.remove(id);

        Ok(())
    }

    fn validate(&self, id: &str, document: &Document) -> Result<(), Error> {
        let collection_ref = match self.collection_by_id.get(id) {
            Some(c) => c,
            None => return Error::code(NOT_FOUND)
                .message("collection not found")
                .throw(),
        };

        let collection = collection_ref.value().read().unwrap();
        if collection.status != CollectionStatus::Ready {
            return Error::code(NOT_READY)
                .message("collection not ready or lagging")
                .throw();
        }

        collection.validate_document(document)
    }
}
