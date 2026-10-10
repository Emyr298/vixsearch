use crate::{common::Document, document::param_result::InsertParam, utils::vixerr::Error};

pub trait DocumentService: Send + Sync {
    fn get_by_id(&self, collection_id: &str, document_id: &str) -> Result<Document, Error>;
    fn get_by_seq_id(&self, collection_id: &str, document_seq_id: &u64) -> Result<Document, Error>;
    fn insert(&self, collection_id: &str, param: InsertParam) -> Result<Document, Error>;
}
