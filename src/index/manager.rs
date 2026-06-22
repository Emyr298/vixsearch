use crate::{index::{GetParam, IndexType, param_result::{CreateParam, DeleteParam, InsertParam, SearchParam, SearchResult, TypeCreateParam}}, shared::{Document, ValueType}, utils::vixerr::Error};

pub trait Manager: Send + Sync {
    fn create(&self, param: CreateParam) -> Result<(), Error>;
    fn delete(&self, param: DeleteParam) -> Result<(), Error>;
    fn get(&self, id: GetParam) -> Result<Document, Error>;
    fn search(&self, param: SearchParam) -> Result<SearchResult, Error>;
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
    fn validate_field(&self, field_type: &ValueType, index_type: IndexType) -> Result<(), Error>;
}

pub trait TypeManager: Send + Sync {
    fn create(&self, param: TypeCreateParam) -> Result<(), Error>;
    fn delete(&self, param: DeleteParam) -> Result<(), Error>;
    fn search(&self, param: SearchParam) -> Result<SearchResult, Error>;
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
}
