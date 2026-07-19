use std::fmt::Display;

use crate::{document::{entity::{DocumentID, DocumentSeqID, SegmentID}, state::{CollectionState, SegmentState}}, shared::Document, utils::vixerr::Error};

pub trait Manager: Send + Sync {
    fn get_by_id(&self, collection_id: &str, document_id: &str) -> Result<Document, Error>;
    fn get_by_seq_id(&self, collection_id: &str, document_seq_id: &u64) -> Result<Document, Error>;
}

pub trait Port: Send + Sync {
    fn get_docs_from_id_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<(DocumentID, Document)>, Error>;
    fn get_docs_from_seq_id_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<(DocumentSeqID, Document)>, Error>;
}

pub trait Lookup: Send + Sync {
    type Key: Display + Ord + Clone;

    fn doc_from_buffer(&self, collection: &CollectionState, key: &Self::Key) -> Option<Document>;
    fn doc_from_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<(Self::Key, Document)>, Error>;
    fn segment_may_contain_doc(&self, segment: &SegmentState, key: &Self::Key) -> bool;
}
