use std::sync::{Arc, RwLock};

use dashmap::{DashMap, Entry};

use crate::{errcode, index::{self, identifier::{DeleteParam, buffer::Buffer, manager::Manager, param_result::{CreateParam, InsertParam, GetParam}}}, query as qry, shared::Document, utils::vixerr::Error};

struct ManagerImpl {
    collection_to_buffers: DashMap<String, Arc<RwLock<Buffer>>>
}

pub fn new_type_manager() -> Box<dyn Manager> {
    Box::new(ManagerImpl{
        collection_to_buffers: DashMap::new(),
    })
}

impl Manager for ManagerImpl {
    fn create(&self, param: CreateParam) -> Result<(), Error> {
        let buffer = Buffer::new();
        
        let buffer_entry = self.collection_to_buffers.entry(param.collection.to_string());
        if let Entry::Occupied(_) = &buffer_entry {
            return Err(Error::new(errcode::EXISTS, "Index already exists"));
        }

        let buffer_arc = Arc::new(RwLock::new(buffer));
        buffer_entry.insert(buffer_arc);
        
        Ok(())
    }
    
    fn get(&self, param: GetParam) -> Result<Document, Error> {
        let buffer_ref = self.collection_to_buffers
            .get(param.collection)
            .ok_or_else(|| Error::new(errcode::FATAL_ERROR, "index not found"))?;

        let buffer_arc = Arc::clone(buffer_ref.value());
        drop(buffer_ref);

        let buffer = buffer_arc.read().unwrap();
        return buffer.get(param.id);
    }
    
    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        let buffer_ref = self.collection_to_buffers
            .get(param.collection)
            .ok_or_else(|| Error::new(errcode::FATAL_ERROR, "index not found"))?;

        let buffer_arc = Arc::clone(buffer_ref.value());
        drop(buffer_ref);

        let buffer = buffer_arc.write().unwrap();
        buffer.insert(param.id, param.document);

        Ok(())
    }

    fn delete(&self, param: DeleteParam) -> Result<(), Error> {
        todo!()
    }
}
