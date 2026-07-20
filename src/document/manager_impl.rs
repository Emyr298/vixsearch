use std::sync::Arc;

use dashmap::DashMap;

use crate::{document::{entity::{CollectionID, Document, DocumentID, DocumentSeqID, SegmentID}, errors::{COLLECTION_NOT_FOUND, DOCUMENT_NOT_FOUND}, lookup_impl::{IDLookup, SeqIDLookup}, manager::{Lookup, Manager, Port}, state::{CollectionState, SegmentState}}, errcode, utils::{vixalg, vixerr::Error}};

pub struct ManagerImpl {
    adapter: Box<dyn Port>,
    collections: DashMap<CollectionID, Arc<CollectionState>>,
    segments: DashMap<SegmentID, Arc<SegmentState>>,
    id_lookup: IDLookup,
    seq_id_lookup: SeqIDLookup,
}

pub fn new_manager(adapter: Box<dyn Port>, id_lookup: IDLookup, seq_id_lookup: SeqIDLookup) -> Box<dyn Manager> {
    Box::new(ManagerImpl::new(adapter, id_lookup, seq_id_lookup))
}

impl ManagerImpl {
    pub fn new(adapter: Box<dyn Port>, id_lookup: IDLookup, seq_id_lookup: SeqIDLookup) -> Self {
        ManagerImpl {
            adapter,
            collections: DashMap::new(),
            segments: DashMap::new(),
            id_lookup,
            seq_id_lookup,
        }
    }
}

impl Manager for ManagerImpl {
    fn get_by_id(&self, collection_id: &CollectionID, document_id: &DocumentID) -> Result<Document, Error> {
        self.get_doc(collection_id, &self.id_lookup, &document_id)
    }

    fn get_by_seq_id(&self, collection_id: &CollectionID, seq_id: &DocumentSeqID) -> Result<Document, Error> {
        self.get_doc(collection_id, &self.seq_id_lookup, &seq_id)
    }

    fn insert(&self, collection_id: &CollectionID, document: Document) -> Result<(), Error> {
        let Some(collection) = self.collections.get(&collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id.0))
                .throw();
        };

        let buffer = collection.buffer.load();
        buffer.id_to_seq_id.insert(document.id.clone(), document.seq_id.clone());
        buffer.buffer.insert(document.seq_id.clone(), document);

        Ok(())
    }

    fn flush(&self, collection_id: &CollectionID) -> Result<(), Error> {
        
    }
}

impl ManagerImpl {
    fn get_doc<LookupT: Lookup>(&self, collection_id: &CollectionID, lookup: &LookupT, key: &LookupT::Key) -> Result<Document, Error> {
        let Some(collection) = self.collections.get(&collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id.0))
                .throw();
        };

        if let Some(doc) = lookup.doc_from_buffer(&collection, key) {
            return Ok(doc);
        }

        for segment_id in collection.get_segment_ids() {
            let Some(segment) = self.segments.get(&segment_id).map(|s| Arc::clone(s.value())) else {
                return Error::code(errcode::FATAL_ERROR)
                    .message(format!("segment {} not found", segment_id.0))
                    .throw();
            };

            if !lookup.segment_may_contain_doc(&segment, key) {
                continue;
            }

            match self.get_doc_from_segment(&segment, lookup, key) {
                Ok(doc) => return Ok(doc),
                Err(err) => {
                    if err.code == DOCUMENT_NOT_FOUND {
                        continue;
                    }
                    return Err(err);
                }
            };
        }

        Error::code(DOCUMENT_NOT_FOUND)
            .message(format!("document {} not found", key))
            .throw()
    }

    fn get_doc_from_segment<LookupT: Lookup>(&self, segment: &SegmentState, lookup: &LookupT, key: &LookupT::Key) -> Result<Document, Error> {
        let Some(doc) = vixalg::binary_search(&segment.id_block_offsets, |block_offset| {
            self.get_doc_from_segment_compare_fn::<LookupT>(segment, lookup, block_offset, key)
        })? else {
            return Error::code(DOCUMENT_NOT_FOUND)
                .message(format!("document {} not found in segment {}", key, segment.id))
                .throw();
        };

        Ok(doc)
    }

    fn get_doc_from_segment_compare_fn<LookupT: Lookup>(&self, segment: &SegmentState, lookup: &LookupT, block_offset: &u64, key: &LookupT::Key) -> Result<vixalg::Ordering<Document>, Error> {
        let docs = lookup.doc_from_block(&segment.id, block_offset)?;
        if docs.len() == 0 {
            return Error::code(errcode::FATAL_ERROR)
                .message(format!("segment {} has no documents in block {}", segment.id.0, block_offset))
                .throw();
        }

        let first_key = lookup.key(&docs[0]);
        if key < first_key {
            return Ok(vixalg::Ordering::Less);
        }

        let last_key = lookup.key(&docs[docs.len() - 1]);
        if key > last_key {
            return Ok(vixalg::Ordering::Greater);
        }

        for doc in docs {
            if lookup.key(&doc) == key {
                return Ok(vixalg::Ordering::Equal(doc));
            }
        }

        Ok(vixalg::Ordering::NotFound)
    }
}