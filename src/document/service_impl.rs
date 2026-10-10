use std::sync::Arc;

use crate::{collection::CollectionLifecycleEvent, common::Document, document::{DocumentService, OpSeqCommitEvent, engine::{self, DOCUMENT_NOT_FOUND, DocumentEngine, OpSeqCommitEngineEvent}, entity::{key_from_id, key_from_seq_id}, param_result::InsertParam}, errcode::FATAL_ERROR, utils::{handler::handler::Handler, observer::observer::{Observer, ObserverGroupReader}, vixerr::Error}};

pub struct DocumentServiceImpl {
    engine: Arc<dyn DocumentEngine>,
    op_seq_commit_observers: Arc<dyn ObserverGroupReader<OpSeqCommitEvent>>,
}

impl DocumentServiceImpl {
    pub fn new(
        engine: Arc<dyn engine::DocumentEngine>,
        op_seq_commit_observers: Arc<dyn ObserverGroupReader<OpSeqCommitEvent>>,
    ) -> Arc<dyn DocumentService> {
        Arc::new(DocumentServiceImpl {
            engine,
            op_seq_commit_observers,
        })
    }
}

impl DocumentService for DocumentServiceImpl {
    fn get_by_id(&self, collection_id: &str, document_id: &str) -> Result<Document, Error> {
        let seq_id = self.get_seq_id_by_id(collection_id, document_id)?;

        let key = key_from_seq_id(&seq_id);
        let doc_raw = self.engine.get_by_key(collection_id, &key)?;
        let doc: Document = rmp_serde::from_slice(&doc_raw).unwrap();
        Ok(doc)
    }

    fn get_by_seq_id(&self, collection_id: &str, document_seq_id: &u64) -> Result<Document, Error> {
        let key = key_from_seq_id(document_seq_id);
        let doc_raw = self.engine.get_by_key(collection_id, &key)?;
        let doc: Document = rmp_serde::from_slice(&doc_raw).unwrap();
        Ok(doc)
    }

    fn insert(&self, collection_id: &str, param: InsertParam) -> Result<Document, Error> {
        let existing_seq_id = match self.get_seq_id_by_id(collection_id, &param.doc_id) {
            Ok(val) => Some(val),
            Err(err) if err.code == DOCUMENT_NOT_FOUND => None,
            Err(err) => return Err(err),
        };

        let op_seq = param.op_seq;
        let doc = param.document(existing_seq_id);
        let doc_bytes = match rmp_serde::to_vec(&doc) {
            Ok(val) => val,
            Err(err) => return Error::code(FATAL_ERROR)
                .message("failed to serialize payload")
                .wrap(err)
                .throw(),
        };

        if existing_seq_id.is_none() {
            self.engine.insert(
                collection_id,
                &key_from_id(&doc.id),
                &doc.seq_id.to_le_bytes(),
                op_seq,
            )?;
        }

        self.engine.insert(
            collection_id,
            &key_from_seq_id(&doc.seq_id),
            &doc_bytes,
            op_seq,
        )?;

        Ok(doc)
    }
}

impl DocumentServiceImpl {
    fn get_seq_id_by_id(&self, collection_id: &str, document_id: &str) -> Result<u64, Error> {
        let key = key_from_id(document_id);

        let seq_id_raw = self.engine.get_by_key(collection_id, &key)?;
        if seq_id_raw.len() != 8 {
            return Error::code(FATAL_ERROR)
                .message("invalid seq_id: length is not 8")
                .throw();
        }

        let seq_id = u64::from_le_bytes(seq_id_raw[..8].try_into().unwrap());
        Ok(seq_id)
    }
}

impl Observer<OpSeqCommitEngineEvent> for DocumentServiceImpl {
    fn observe(&self, engine_event: &OpSeqCommitEngineEvent) {
        let event = OpSeqCommitEvent::new(engine_event);
        for observer in self.op_seq_commit_observers.list().iter() {
            observer.observe(&event);
        }
    }
}
