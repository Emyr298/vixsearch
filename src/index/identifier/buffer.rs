use dashmap::{DashMap, Entry};

use crate::{errcode, index::{self, identifier::param_result::{document_result}, layered::Buffer}, query as qry, shared::{Document, Value}, utils::vixerr::Error};

pub struct IdentifierBuffer {
    buffer_map: DashMap<String, Document>
}

impl IdentifierBuffer {
    pub fn new() -> Self {
        IdentifierBuffer{
            buffer_map: DashMap::new()
        }
    }
}

// TODO: segment lookup
// TODO: constraint checking, but may be outside of this insert function (before translog)
impl Buffer for IdentifierBuffer {
    fn search(&self, param: Box<dyn qry::SearchQuery>) -> Result<index::SearchResult, Error> {
        let query = param
            .into_any()
            .downcast::<qry::EqualityQuery>()
            .map_err(|_| Error::new(errcode::SYSTEM_ERROR, "invalid query"))?;

        let Value::String(id) = query.value else {
            return Err(Error::new(errcode::ARGUMENT_ERROR, "id must be a string"));
        };

        let doc_ref = self.buffer_map
            .get(&id)
            .ok_or_else(|| Error::new(errcode::NOT_FOUND, "document not found"))?;

        let doc = doc_ref.clone();
        Ok(document_result(doc))
    }

    fn insert(&self, param_lookup: Box<dyn index::Lookup>, param_entry: Box<dyn index::Entry>) -> Result<(), Error> {
        let id = param_lookup
            .into_any()
            .downcast::<String>()
            .map_err(|_| Error::new(errcode::SYSTEM_ERROR, "invalid query"))?;

        let document = param_entry
            .into_any()
            .downcast::<Document>()
            .map_err(|_| Error::new(errcode::SYSTEM_ERROR, "invalid query"))?;

        // Note: id is expected to be locked already, so there won't be race condition of insertion
        let doc_entry = self.buffer_map.entry(*id);
        if let Entry::Occupied(_) = &doc_entry {
            return Err(Error::new(errcode::EXISTS, "Document with same id already exists"));
        }

        doc_entry.insert(*document);
        Ok(())
    }

    fn flush(&self) {
        todo!()
    }
}
