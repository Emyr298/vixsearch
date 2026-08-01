use crate::{document::{engine::{self, DOCUMENT_NOT_FOUND}, entity::{Document, key_from_id, key_from_seq_id}, manager::Manager, param_result::InsertParam}, errcode::FATAL_ERROR, utils::vixerr::Error};

pub struct ManagerImpl {
    engine: Box<dyn engine::Engine>,
}

pub fn new_manager(engine: Box<dyn engine::Engine>) -> Box<dyn Manager> {
    Box::new(ManagerImpl::new(engine))
}

impl ManagerImpl {
    pub fn new(engine: Box<dyn engine::Engine>) -> Self {
        ManagerImpl {
            engine,
        }
    }
}

impl Manager for ManagerImpl {
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

    fn flush(&self, collection_id: &str) -> Result<(), Error> {
        self.engine.flush(collection_id)
    }
}

impl ManagerImpl {
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
