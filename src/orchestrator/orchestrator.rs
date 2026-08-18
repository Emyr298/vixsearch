use crate::{orchestrator::CreateCollectionParam, shared::Document, vixerr::Error};

pub trait Orchestrator: Send + Sync {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error>;
    fn delete_collection(&self, name: &str) -> Result<(), Error>;
    fn load_collection(&self) -> Result<(), Error>;

    fn get_document(&self, collection: &str, id: &str) -> Result<Document, Error>;
    fn insert_document(&self, collection: String, document: Document) -> Result<(), Error>;
}
