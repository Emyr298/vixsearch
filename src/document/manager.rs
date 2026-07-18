use crate::{document::state::DocumentID, shared::Document, utils::vixerr::Error};

pub trait Manager: Send + Sync {
    fn get_by_id(&self, collection_id: &str, document_id: &str) -> Result<Document, Error>;
}

pub trait Port: Send + Sync {
    fn get_docs_from_block(&self, segment_id: &str, block_offset: u64) -> Result<Vec<(DocumentID, Document)>, Error>;
}
