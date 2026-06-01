use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use dashmap::{DashMap, Entry};

use crate::collection::CollectionInstance;
use crate::collection::entity::Collection;
use crate::collection::instance::InstanceStatus;
use crate::collection::param_result::{CreateCollectionParam, CreateStorageParam};
use crate::vixerr::Error;
use crate::{errcode, shared};

pub trait Storage: Send + Sync {
    fn create(&self, param: CreateStorageParam) -> Result<(), Error>;
    fn delete(&self, id: &str) -> Result<(), Error>;
    fn load(&self) -> Result<Vec<Collection>, Error>;
}

pub trait Manager: Send + Sync {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error>;
    fn delete_collection(&self, name: &str) -> Result<(), Error>;
    fn load(&self) -> Result<(), Error>;
    fn validate_payload(
        &self,
        collection_name: &str,
        payload: &HashMap<String, shared::Value>,
    ) -> Result<bool, Error>;
}

struct ManagerImpl {
    storage: Box<dyn Storage>,
    instances: DashMap<String, Arc<RwLock<CollectionInstance>>>,
}

pub fn new_manager(storage: Box<dyn Storage>) -> Box<dyn Manager> {
    return Box::new(ManagerImpl::new(storage));
}

impl ManagerImpl {
    fn new(storage: Box<dyn Storage>) -> Self {
        return ManagerImpl {
            storage,
            instances: DashMap::new(),
        };
    }
}

impl Manager for ManagerImpl {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error> {
        let id = uuid::Uuid::new_v4().to_string();

        let instance_entry = self.instances.entry(param.name.clone());
        if let Entry::Occupied(_) = &instance_entry {
            return Err(Error::new(errcode::EXISTS, "Collection already exists"));
        }

        let instance_arc = Arc::new(RwLock::new(CollectionInstance::new(
            &id,
            &param.name,
            &param.fields,
            InstanceStatus::Loading,
        )));

        instance_entry.insert(Arc::clone(&instance_arc)); // early drop for dashmap shard, so it doesn't need to wait for File I/O to release

        let param_name = param.name.clone();
        if let Err(e) = self.storage.create(param.into_create_storage_param(id)) {
            self.instances.remove(&param_name);
            return Err(e);
        }

        let mut instance = instance_arc.write().unwrap();
        instance.status = InstanceStatus::Ready;

        Ok(())
    }

    fn delete_collection(&self, name: &str) -> Result<(), Error> {
        let instance_ref = self
            .instances
            .get(name)
            .ok_or_else(|| Error::new(errcode::NOT_FOUND, "Collection not found"))?;

        let instance_arc = Arc::clone(instance_ref.value());
        drop(instance_ref);

        let mut instance = instance_arc.write().unwrap();
        if instance.status == InstanceStatus::Deleting {
            return Ok(());
        }

        // TODO: check if atomic or not -> if not, should consider partial deletion
        if let Err(e) = self.storage.delete(&instance.id) {
            return Err(e);
        }

        instance.status = InstanceStatus::Deleting;

        self.instances.remove(name);

        Ok(())
    }

    fn validate_payload(
        &self,
        collection_name: &str,
        document: &HashMap<String, shared::Value>,
    ) -> Result<bool, Error> {
        let instance_ref = self
            .instances
            .get(collection_name)
            .ok_or_else(|| Error::new(errcode::NOT_FOUND, "Collection not found"))?;

        let instance = instance_ref.value().read().unwrap();

        if instance.status != InstanceStatus::Ready {
            return Err(Error::new(
                errcode::NOT_READY,
                "Collection not ready or lagging",
            ));
        }

        Ok(instance.validate_document(document))
    }
    
    fn load(&self) -> Result<(), Error> {
        let colls = self.storage.load()?;
        for coll in colls {
            let instance = CollectionInstance::from(coll);
            self.instances.insert(instance.name.clone(), Arc::new(RwLock::new(instance)));
        }
        Ok(())
    }
}
