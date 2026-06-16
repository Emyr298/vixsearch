use crate::{errcode, index::{self, identifier::buffer::IdentifierBuffer, layered}, query as qry, shared::Document, utils::vixerr::Error};

struct TypeManagerImpl {
    base: layered::BaseTypeManager<IdentifierBuffer>,
}

pub fn new_type_manager() -> impl index::TypeManager {
    TypeManagerImpl{
        base: layered::BaseTypeManager::new(),
    }
}

impl index::TypeManager for TypeManagerImpl {
    fn create(&self, param: index::TypeCreateParam) -> Result<(), Error> {
        let buffer = IdentifierBuffer::new();
        self.base.create(param.collection, param.field, buffer)
    }
    
    fn delete(&self, param: index::DeleteParam) -> Result<(), Error> {
        todo!()
    }
    
    fn search(&self, param: index::SearchParam) -> Result<index::SearchResult, Error> {
        self.base.search(param)
    }
    
    fn insert(&self, param: index::InsertParam) -> Result<(), Error> {
        self.base.insert(param)
    }
}
