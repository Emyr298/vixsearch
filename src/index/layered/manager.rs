use std::{sync::{Arc, RwLock}, time::Duration};

use dashmap::DashMap;

use crate::{errcode, index::layered::{buffer::Buffer, entity::buffer_key, param_result::{FlushParam, InsertParam}}, utils::{vixerr::Error, vixscheduler::Scheduler}};

pub struct Base<ParamT, ResultT> {
    buffers: DashMap<String, Arc<RwLock<dyn Buffer<ParamT, ResultT>>>>
}

impl<ParamT, ResultT> Base<ParamT, ResultT> {
    pub fn new() -> Self {
        Base {
            buffers: DashMap::new(),
        }
    }

    pub fn insert(&self, param: InsertParam) -> Result<(), Error> {
        let key: String = buffer_key(&param.collection, &param.field);
        let buffer_ref = self.buffers.get(&key).ok_or_else(|| Error::new(errcode::FATAL_ERROR, "index not found"))?;

        let buffer_arc = Arc::clone(buffer_ref.value());
        drop(buffer_ref);

        let buffer = buffer_arc.write().unwrap();
        buffer.insert(&param.id, &param.value);

        Ok(())
    }

    pub fn flush(&self, param: FlushParam) -> Result<(), Error> {
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
