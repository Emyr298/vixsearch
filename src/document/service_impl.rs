use std::sync::Arc;

use crate::{document::{DocumentCollectionLifecycle, DocumentService, engine::{self, DOCUMENT_NOT_FOUND, DocumentEngine, DocumentEngineCollectionLifecycle}, entity::{Document, key_from_id, key_from_seq_id}, param_result::InsertParam}, errcode::FATAL_ERROR, utils::vixerr::Error};

pub struct DocumentServiceImpl {
    engine: Arc<dyn DocumentEngine>,
    engine_collection_lifecycle: Arc<dyn DocumentEngineCollectionLifecycle>
}

impl DocumentServiceImpl {
    pub fn new(engine: Arc<dyn engine::DocumentEngine>, engine_loader: Arc<dyn DocumentEngineCollectionLifecycle>) -> (Arc<dyn DocumentService>, Arc<dyn DocumentCollectionLifecycle>) {
        let arc = Arc::new(DocumentServiceImpl {
            engine,
            engine_collection_lifecycle: engine_loader,
        });

        let service_arc: Arc<dyn DocumentService> = arc.clone();
        let lifecycle_arc: Arc<dyn DocumentCollectionLifecycle> = arc;

        (service_arc, lifecycle_arc)
    }
}

impl DocumentCollectionLifecycle for DocumentServiceImpl {
    fn load_collections(&self, collection_ids: &[String]) -> Result<(), Error> {
        self.engine_collection_lifecycle.load_collections(collection_ids)
    }
    
    fn add_collection(&self, collection_id: &str) -> Result<(), Error> {
        self.engine_collection_lifecycle.add_collection(collection_id)
    }
    
    fn delete_collection(&self, collection_id: &str) -> Result<(), Error> {
        self.engine_collection_lifecycle.delete_collection(collection_id)
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

    fn insert(&self, collection_id: &str, param: InsertParam) -> Result<(), Error> {
        let existing_seq_id = match self.get_seq_id_by_id(collection_id, &param.id) {
            Ok(val) => Some(val),
            Err(err) if err.code == DOCUMENT_NOT_FOUND => None,
            Err(err) => return Err(err),
        };

        let doc = match rmp_serde::to_vec(&param.payload) {
            Ok(val) => val,
            Err(err) => return Error::code(FATAL_ERROR)
                .message("failed to serialize payload")
                .wrap(err)
                .throw(),
        };

        let seq_id = match existing_seq_id {
            Some(id) => id,
            None => {
                self.engine.insert(
                    collection_id,
                    &key_from_id(&param.id),
                    &param.op_seq.to_le_bytes()
                )?;
                param.op_seq
            }
        };

        self.engine.insert(
            collection_id,
            &key_from_seq_id(&seq_id),
            &doc
        )?;

        Ok(())
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
