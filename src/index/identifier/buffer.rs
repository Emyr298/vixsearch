use dashmap::{DashMap, Entry};

use crate::{errcode, index, query as qry, shared::{Document, Value}, utils::vixerr::Error};

pub struct Buffer {
    buffer_map: DashMap<String, Document>
}

impl Buffer {
    pub fn new() -> Self {
        Buffer{
            buffer_map: DashMap::new()
        }
    }
}

// TODO: segment lookup
// TODO: constraint checking, but may be outside of this insert function (before translog, when translog is added ofc)
impl Buffer {
    pub fn get(&self, id: &str) -> Result<Document, Error> {
        let doc_ref = self.buffer_map
            .get(id)
            .ok_or_else(|| Error::new(errcode::NOT_FOUND, "document not found"))?;

        let doc = doc_ref.clone();
        Ok(doc)
    }

    // id is expected to be locked when using this function
    pub fn insert(&self, id: &str, document: &Document) -> Result<(), Error> {
        // as id is locked, there won't be race condition between validation and insert
        if !matches!(self.get(id), Err(e) if e.code == errcode::NOT_FOUND) {
            return Err(Error::new(errcode::EXISTS, "Document with same id already exists"));
        }

        self.buffer_map.insert(id.to_string(), document.clone());
        Ok(())
    }

    pub fn flush(&self) {
        
    }
}
