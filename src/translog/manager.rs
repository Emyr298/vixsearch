use crate::{query, vixerr::Error};

// TODO: sharding
pub trait Manager: Send + Sync {
    fn write_insert(&self, collection: String, operation: query::InsertData) -> Result<(), Error>;
}

pub struct MangerImpl {}

pub fn new_manager() -> Box<dyn Manager> {
    Box::new(MangerImpl::new())
}

impl MangerImpl {
    pub fn new() -> Self {
        MangerImpl {}
    }
}

impl Manager for MangerImpl {
    fn write_insert(
        &self,
        _collection: String,
        _operation: query::InsertData,
    ) -> Result<(), Error> {
        todo!()
    }
}
