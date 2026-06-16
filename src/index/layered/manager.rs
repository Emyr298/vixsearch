use std::{marker::PhantomData, sync::{Arc, RwLock}};

use dashmap::{DashMap, Entry};

use crate::{errcode, index::{self, layered::{buffer::Buffer, entity::buffer_key}}, utils::vixerr::Error};

pub struct BaseTypeManager<BufferT> where BufferT: Buffer {
    buffers: DashMap<String, Arc<RwLock<BufferT>>>,
}

impl<BufferT> BaseTypeManager<BufferT> where BufferT: Buffer {
    pub fn new() -> Self {
        BaseTypeManager {
            buffers: DashMap::new(),
        }
    }

    pub fn create(&self, collection: String, field: String, buffer: BufferT) -> Result<(), Error> {
        let key: String = buffer_key(&collection, &field);
        let buffer_entry = self.buffers.entry(key);
        if let Entry::Occupied(_) = &buffer_entry {
            return Err(Error::new(errcode::EXISTS, "Index already exists"));
        }

        let buffer_arc = Arc::new(RwLock::new(buffer));
        buffer_entry.insert(buffer_arc);
        
        Ok(())
    }

    pub fn search(&self, param: index::SearchParam) -> Result<index::SearchResult, Error> {
        let key: String = buffer_key(&param.collection, &param.field);
        let buffer_ref = self.buffers.get(&key).ok_or_else(|| Error::new(errcode::FATAL_ERROR, "index not found"))?;

        let buffer_arc = Arc::clone(buffer_ref.value());
        drop(buffer_ref);

        let buffer = buffer_arc.read().unwrap();
        return buffer.search(param.query);
    }

    pub fn insert(&self, param: index::InsertParam) -> Result<(), Error> {
        let key: String = buffer_key(&param.collection, &param.field);
        let buffer_ref = self.buffers.get(&key).ok_or_else(|| Error::new(errcode::FATAL_ERROR, "index not found"))?;

        let buffer_arc = Arc::clone(buffer_ref.value());
        drop(buffer_ref);

        let buffer = buffer_arc.write().unwrap();
        buffer.insert(param.lookup, param.entry);

        Ok(())
    }

    pub fn flush(&self, param: index::FlushParam) -> Result<(), Error> {
        let key: String = buffer_key(&param.collection, &param.field);
        let buffer_ref = self.buffers.get(&key).ok_or_else(|| Error::new(errcode::FATAL_ERROR, "index not found"))?;

        let buffer_arc = Arc::clone(buffer_ref.value());
        drop(buffer_ref);

        let buffer = buffer_arc.write().unwrap();
        buffer.flush();

        Ok(())
    }

    pub fn merge(&self) {
        todo!()
    }
}
