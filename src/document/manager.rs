use crate::{document::entity::Document, utils::vixerr::Error};

pub trait Manager: Send + Sync {
    fn get_by_id(&self, collection_id: &str, document_id: &str) -> Result<Document, Error>;
    fn get_by_seq_id(&self, collection_id: &str, document_seq_id: &str) -> Result<Document, Error>;
    fn insert(&self, collection_id: &str, document: Document) -> Result<(), Error>;
    fn flush(&self, collection_id: &str) -> Result<(), Error>;
}
