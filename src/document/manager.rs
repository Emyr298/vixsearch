use std::fmt::Display;

use crate::{document::{entity::{CollectionID, Document, DocumentID, DocumentSeqID, SegmentID}, state::{CollectionState, SegmentState}}, utils::vixerr::Error};

pub trait Manager: Send + Sync {
    fn get_by_id(&self, collection_id: &CollectionID, document_id: &DocumentID) -> Result<Document, Error>;
    fn get_by_seq_id(&self, collection_id: &CollectionID, document_seq_id: &DocumentSeqID) -> Result<Document, Error>;
    fn insert(&self, collection_id: &CollectionID, document: Document) -> Result<(), Error>;
    fn flush(&self, collection_id: &CollectionID) -> Result<(), Error>;
}

pub trait Port: Send + Sync {
    fn get_from_id_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<Document>, Error>;
    fn get_from_seq_id_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<Document>, Error>;
    fn flush_segment(&self) -> Result<(), Error>;
}

pub trait Lookup: Send + Sync {
    type Key: Display + Ord + Clone;

    fn key<'a>(&self, doc: &'a Document) -> &'a Self::Key;
    fn doc_from_buffer(&self, collection: &CollectionState, key: &Self::Key) -> Option<Document>;
    fn doc_from_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<Document>, Error>;
    fn segment_may_contain_doc(&self, segment: &SegmentState, key: &Self::Key) -> bool;
}
