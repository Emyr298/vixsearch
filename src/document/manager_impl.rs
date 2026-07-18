use std::sync::Arc;

use dashmap::DashMap;

use crate::{document::{errors::{COLLECTION_NOT_FOUND, DOCUMENT_NOT_FOUND}, manager::{Manager, Port}, state::{CollectionID, CollectionState, DocumentID, SegmentID, SegmentState}}, errcode, shared::Document, utils::{vixalg, vixerr::Error}};

pub struct ManagerImpl {
    adapter: Box<dyn Port>,
    collections: DashMap<CollectionID, Arc<CollectionState>>,
    segments: DashMap<SegmentID, Arc<SegmentState>>,
}

pub fn new_manager(adapter: Box<dyn Port>) -> Box<dyn Manager> {
    Box::new(ManagerImpl::new(adapter))
}

impl ManagerImpl {
    pub fn new(adapter: Box<dyn Port>) -> Self {
        ManagerImpl {
            adapter,
            collections: DashMap::new(),
            segments: DashMap::new(),
        }
    }

    fn get_doc_from_segment(&self, segment: &SegmentState, document_id: &DocumentID) -> Result<Document, Error> {
        let Some(doc) = vixalg::binary_search(&segment.block_offsets, |block_offset| {
            self.get_doc_from_segment_compare_fn(segment, block_offset, document_id)
        })? else {
            return Error::code(DOCUMENT_NOT_FOUND)
                .message(format!("document {} not found in segment {}", document_id.0, segment.id.0))
                .throw();
        };

        Ok(doc)
    }

    fn get_doc_from_segment_compare_fn(&self, segment: &SegmentState, block_offset: &u64, document_id: &DocumentID) -> Result<vixalg::Ordering<Document>, Error> {
        let docs = self.adapter.get_docs_from_block(&segment.id.0, *block_offset)?;
        if docs.len() == 0 {
            return Error::code(errcode::FATAL_ERROR)
                .message(format!("segment {} has no documents in block {}", segment.id.0, block_offset))
                .throw();
        }

        let first_doc = &docs[0];
        if document_id.0 < first_doc.0.0 {
            return Ok(vixalg::Ordering::Less);
        }

        let last_doc = &docs[docs.len() - 1];
        if document_id.0 > last_doc.0.0 {
            return Ok(vixalg::Ordering::Greater);
        }

        for (doc_id, doc) in docs {
            if doc_id == *document_id {
                return Ok(vixalg::Ordering::Equal(doc));
            }
        }

        Ok(vixalg::Ordering::NotFound)
    }
}

impl Manager for ManagerImpl {
    fn get_by_id(&self, collection_id: &str, document_id: &str) -> Result<Document, Error> {
        let collection_id = CollectionID(collection_id.to_string());
        let document_id = DocumentID(document_id.to_string());

        let Some(collection) = self.collections.get(&collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id.0))
                .throw();
        };

        if let Some(doc) = collection.doc_from_buffer(&document_id) {
            return Ok(doc);
        }

        let levels = collection.levels.load();
        if levels.len() == 0 {
            return Error::code(DOCUMENT_NOT_FOUND)
                .message(format!("document {} not found", document_id.0))
                .throw();
        }

        for segment_id in collection.get_segment_ids() {
            let Some(segment) = self.segments.get(&segment_id).map(|s| Arc::clone(s.value())) else {
                return Error::code(errcode::FATAL_ERROR)
                    .message(format!("segment {} not found", segment_id.0))
                    .throw();
            };

            if document_id.0 < segment.smallest_id.0 || document_id.0 > segment.biggest_id.0 {
                continue;
            }

            if !segment.id_filter.contains(&document_id.0) {
                continue;
            }

            match self.get_doc_from_segment(&segment, &document_id) {
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
            .message(format!("document {} not found", document_id.0))
            .throw()
    }
}
