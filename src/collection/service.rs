use crate::collection::port_param_result::{CreatePortParam, GetAllPortResult};
use crate::collection::service_param_result::CreateParam;
use crate::document::Document;
use crate::vixerr::Error;

pub trait CollectionService: Send + Sync {
    fn create(&self, param: CreateParam) -> Result<(), Error>;
    fn delete_by_id(&self, id: &str) -> Result<(), Error>;
    fn validate_by_id(&self, collection_name: &str, document: &Document) -> Result<(), Error>;
}

pub trait CollectionLoader: Send + Sync {
    fn load(&self) -> Result<(), Error>;
}

pub trait CollectionPort: Send + Sync {
    fn get_all(&self) -> Result<GetAllPortResult, Error>;
    fn create(&self, param: CreatePortParam) -> Result<(), Error>;
    fn delete_by_internal_id(&self, id: &str) -> Result<(), Error>;
}
