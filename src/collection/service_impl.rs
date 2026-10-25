use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use dashmap::{DashMap, Entry};
use uuid::Uuid;

use crate::{
    collection::{
        GetByIDResult,
        entity::{
            CollectionLifecycleEvent, CollectionLifecycleEventCreated,
            CollectionLifecycleEventDeleted, CollectionLifecycleEventLoaded,
        },
        service::{CollectionLoader, CollectionPort, CollectionService},
        service_param_result::CreateParam,
        state::{CollectionState, CollectionStatus, collection_ids},
    },
    common::{RawValue, Value},
    errcode::{EXISTS, NOT_FOUND, NOT_READY},
    utils::{handler::handler::HandlerGroupReader, vixerr::Error},
};

pub struct ServiceImpl {
    adapter: Arc<dyn CollectionPort>,

    lifecycle_handlers: Arc<dyn HandlerGroupReader<CollectionLifecycleEvent>>,

    // in the future, when decide to add alter collection, should reassess rwlock since it will block read mid traffic
    collection_by_id: DashMap<String, Arc<RwLock<CollectionState>>>,
    collection_by_internal_id: DashMap<String, Arc<RwLock<CollectionState>>>,
}

impl ServiceImpl {
    pub fn new(
        adapter: Arc<dyn CollectionPort>,
        lifecycle_handlers: Arc<dyn HandlerGroupReader<CollectionLifecycleEvent>>,
    ) -> (Arc<dyn CollectionService>, Arc<dyn CollectionLoader>) {
        let arc = Arc::new(ServiceImpl {
            adapter,
            lifecycle_handlers,
            collection_by_id: DashMap::new(),
            collection_by_internal_id: DashMap::new(),
        });

        let service_arc: Arc<dyn CollectionService> = arc.clone();
        let loader_arc: Arc<dyn CollectionLoader> = arc;

        (service_arc, loader_arc)
    }
}

impl CollectionLoader for ServiceImpl {
    fn load(&self) -> Result<(), Error> {
        let result = self.adapter.get_all()?;

        let collection_states: Vec<(String, String, Arc<RwLock<CollectionState>>)> = result
            .collections
            .iter()
            .map(|c| {
                CollectionState::from_get_all_port_result_collection(c, CollectionStatus::Loading)
            })
            .map(|c| {
                (
                    c.id.clone(),
                    c.internal_id.clone(),
                    Arc::new(RwLock::new(c)),
                )
            })
            .collect();

        for (id, internal_id, collection_state) in &collection_states {
            self.collection_by_id
                .insert(id.to_string(), collection_state.clone());
            self.collection_by_internal_id
                .insert(internal_id.to_string(), collection_state.clone());
        }

        // Sync to other services
        let event = CollectionLifecycleEvent::Loaded(CollectionLifecycleEventLoaded {
            collection_ids: collection_ids(&collection_states),
        });

        for handler in self.lifecycle_handlers.list().iter() {
            handler.handle(&event)?;
        }

        // Mark as ready after sync
        for (_, _, collection_state) in &collection_states {
            let mut collection_write = collection_state.write().unwrap();
            collection_write.status = CollectionStatus::Ready;
        }

        Ok(())
    }
}

impl CollectionService for ServiceImpl {
    fn get_by_id(&self, id: &str) -> Result<GetByIDResult, Error> {
        let state_ref = match self.collection_by_id.get(id) {
            Some(c) => c,
            None => {
                return Error::code(NOT_FOUND)
                    .message("collection not found")
                    .throw();
            }
        };

        let state = state_ref.read().unwrap();
        Ok(GetByIDResult::from_state(&*state))
    }

    fn create(&self, param: CreateParam) -> Result<(), Error> {
        param.validate()?;

        let collection_entry = self.collection_by_id.entry(param.id.to_string());
        if let Entry::Occupied(_) = &collection_entry {
            return Error::code(EXISTS)
                .message("collection already exists")
                .throw();
        }

        let internal_id = Uuid::new_v4().to_string();
        let collection_arc = Arc::new(RwLock::new(CollectionState::new(
            &param.id,
            &internal_id,
            &param.fields,
            CollectionStatus::Loading,
        )));

        collection_entry.insert(collection_arc.clone()); // early drop for dashmap shard, so it doesn't need to wait
        self.collection_by_internal_id
            .insert(internal_id.clone(), collection_arc.clone());

        // Sync to other services
        let event = CollectionLifecycleEvent::Created(CollectionLifecycleEventCreated {
            collection_id: param.id.to_string(),
        });

        for handler in self.lifecycle_handlers.list().iter() {
            handler.handle(&event)?;
        }

        if let Err(e) = self.adapter.create(param.create_port_param(&internal_id)) {
            self.collection_by_id.remove(&param.id);
            self.collection_by_internal_id.remove(&internal_id);
            return Err(e);
        }

        let mut collection = collection_arc.write().unwrap();
        collection.status = CollectionStatus::Ready;

        Ok(())
    }

    fn delete_by_id(&self, id: &str) -> Result<(), Error> {
        let collection_arc = match self.collection_by_id.get(id) {
            Some(c) => c.clone(),
            None => {
                return Error::code(NOT_FOUND)
                    .message("collection not found")
                    .throw();
            }
        };

        let internal_id = {
            let collection = collection_arc.write().unwrap();
            if collection.status == CollectionStatus::Deleting {
                return Ok(());
            }
            collection.internal_id.clone()
        };

        if let Err(e) = self.adapter.delete_by_internal_id(&internal_id) {
            return Err(e);
        }

        // Sync to other services
        let event = CollectionLifecycleEvent::Deleted(CollectionLifecycleEventDeleted {
            collection_id: id.to_string(),
        });

        for handler in self.lifecycle_handlers.list().iter() {
            handler.handle(&event)?;
        }

        self.collection_by_id.remove(id);
        self.collection_by_internal_id.remove(&internal_id);

        Ok(())
    }

    fn parse_raw_document_payload_by_id(
        &self,
        id: &str,
        raw_document_payload: &HashMap<String, RawValue>,
    ) -> Result<HashMap<String, Value>, Error> {
        let collection_arc = match self.collection_by_id.get(id) {
            Some(c) => c.clone(),
            None => {
                return Error::code(NOT_FOUND)
                    .message("collection not found")
                    .throw();
            }
        };

        let collection = collection_arc.read().unwrap();
        if collection.status != CollectionStatus::Ready {
            return Error::code(NOT_READY)
                .message("collection not ready or lagging")
                .throw();
        }

        collection.parse_raw_document_payload(raw_document_payload)
    }
}
