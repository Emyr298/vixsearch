use crate::{document::{engine, entity::{Document, key_from_id, key_from_seq_id}, manager::Manager}, utils::vixerr::Error};

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

        let key = key_from_seq_id(seq_id.as_str());
        let doc_raw = self.engine.get_by_key(collection_id, key.as_str())?;
        let doc: Document = rmp_serde::from_slice(&doc_raw).unwrap();
        Ok(doc)
    }

    fn get_by_seq_id(&self, collection_id: &str, document_seq_id: &str) -> Result<Document, Error> {
        let key = key_from_seq_id(document_seq_id);
        let doc_raw = self.engine.get_by_key(collection_id, key.as_str())?;
        let doc: Document = rmp_serde::from_slice(&doc_raw).unwrap();
        Ok(doc)
    }

    fn insert(&self, _: &str, _: Document) -> Result<(), Error> {
        todo!() // TODO: atomic seq_id:doc and id:seq_id
    }

    fn flush(&self, collection_id: &str) -> Result<(), Error> {
        self.engine.flush(collection_id)
    }
}

impl ManagerImpl {
    fn get_seq_id_by_id(&self, collection_id: &str, document_id: &str) -> Result<String, Error> {
        let key = key_from_id(document_id);
        let seq_id_raw = self.engine.get_by_key(collection_id, key.as_str())?;
        let seq_id = String::from_utf8(seq_id_raw).unwrap();
        Ok(seq_id)
    }
}
