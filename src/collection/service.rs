use crate::collection::port_param_result::{CreatePortParam, GetAllPortResult};
use crate::collection::service_param_result::CreateParam;
use crate::document::Document;
use crate::vixerr::Error;

pub trait Service: Send + Sync {
    fn create(&self, param: CreateParam) -> Result<(), Error>;
    fn delete(&self, id: &str) -> Result<(), Error>;
    fn validate(&self, collection_name: &str, document: &Document) -> Result<(), Error>;
}

pub trait Loader: Send + Sync {
    fn load(&self) -> Result<(), Error>;
}

pub trait Port: Send + Sync {
    fn get_all(&self) -> Result<GetAllPortResult, Error>;
    fn create(&self, param: CreatePortParam) -> Result<(), Error>;
    fn delete(&self, id: &str) -> Result<(), Error>;
}
