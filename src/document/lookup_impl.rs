use crate::{document::{entity::{DocumentID, DocumentSeqID, SegmentID}, manager::{Lookup, Port}, state::{CollectionState, SegmentState}}, shared::Document, utils::vixerr::Error};

pub struct IDLookup {
    adapter: Box<dyn Port>,
}

impl Lookup for IDLookup {
    type Key = DocumentID;

    fn doc_from_buffer(&self, collection: &CollectionState, key: &Self::Key) -> Option<Document> {
        collection.doc_from_buffer_by_id(key)
    }

    fn doc_from_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<(Self::Key, Document)>, Error> {
        self.adapter.get_docs_from_id_block(segment_id, block_offset)
    }

    fn segment_may_contain_doc(&self, segment: &SegmentState, key: &Self::Key) -> bool {
        segment.may_contain_doc_id(key)
    }
}

pub struct SeqIDLookup {
    adapter: Box<dyn Port>,
}

impl Lookup for SeqIDLookup {
    type Key = DocumentSeqID;

    fn doc_from_buffer(&self, collection: &CollectionState, key: &Self::Key) -> Option<Document> {
        collection.doc_from_buffer_by_seq_id(key)
    }

    fn doc_from_block(&self, segment_id: &SegmentID, block_offset: &u64) -> Result<Vec<(Self::Key, Document)>, Error> {
        self.adapter.get_docs_from_seq_id_block(segment_id, block_offset)
    }

    fn segment_may_contain_doc(&self, segment: &SegmentState, key: &Self::Key) -> bool {
        segment.may_contain_doc_seq_id(key)
    }
}
