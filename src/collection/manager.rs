use std::sync::{Arc, RwLock};
use std::{any::Any, collections::HashMap, fs::File, io::Write, path::PathBuf};

use dashmap::DashMap;

use crate::collection::{Collection, CollectionInstance, param_result::CreateCollectionParam};
use crate::vixerr::Error;
use crate::{errcode, shared};

pub trait Manager: Send + Sync {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error>;
    fn delete_collection(&self, name: &str) -> Result<(), Error>;
    fn validate_payload(
        &self,
        collection_name: &str,
        payload: &HashMap<String, shared::Value>,
    ) -> Result<bool, Error>;
}

struct ManagerImpl {
    data_dir: String,
    instances: DashMap<String, Arc<RwLock<CollectionInstance>>>,
}

pub fn new_manager(data_dir: &str) -> Box<dyn Manager> {
    return Box::new(ManagerImpl::new(data_dir));
}

impl ManagerImpl {
    fn new(data_dir: &str) -> Self {
        return ManagerImpl {
            data_dir: data_dir.to_string(),
            instances: DashMap::new(),
        };
    }
}

// TODO: mutex
impl Manager for ManagerImpl {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error> {
        let collection = Collection {
            id: uuid::Uuid::new_v4().to_string(),
            name: param.name,
            fields: param.fields,
        };

        let collection_json = serde_json::to_string(&collection)
            .map_err(|_| Error::new(errcode::SYSTEM_ERROR, "Failed to serialize collection"))?;

        let hash = crc32fast::hash(&collection_json.as_bytes());

        let file_content = format!("{:08x}:{}", hash, collection_json);

        let path = PathBuf::from(self.data_dir.as_str()).join(collection.id);
        let mut file = File::create(path).expect("PANIC: Failed to create collection file on disk");
        file.write_all(file_content.as_bytes())
            .expect("PANIC: Failed to write to collection file on disk");

        Ok(())
    }

    fn delete_collection(&self, id: &str) -> Result<(), Error> {
        let path = PathBuf::from(self.data_dir.as_str()).join(id);
        std::fs::remove_file(path).expect("PANIC: Failed to remove collection file on disk");

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

        let instance = instance_ref
            .value()
            .read()
            .map_err(|_| Error::new(errcode::SYSTEM_ERROR, "Failed to read collection"))?;

        Ok(instance.validate_document(document))
    }
}
