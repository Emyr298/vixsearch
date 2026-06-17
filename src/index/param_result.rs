use std::any::Any;

use crate::{index::IndexType, query, shared::ValueType};

pub struct CreateParam {
    pub collection: String,
    pub field: String,
    pub index_type: IndexType,
}

impl Into<TypeCreateParam> for CreateParam {
    fn into(self) -> TypeCreateParam {
        TypeCreateParam { collection: self.collection, field: self.field }
    }
}

pub struct TypeCreateParam {
    pub collection: String,
    pub field: String,
}

pub struct DeleteParam {
    pub collection: String,
    pub field: String,
}

pub struct SearchParam {
    pub collection: String,
    pub field: String,
    pub query: Box<dyn query::SearchQuery>,
}

pub struct InsertParam {
    pub collection: String,
    pub field: String,
    pub lookup: Box<dyn Lookup>,
    pub entry: Box<dyn Entry>,
}

pub trait Lookup {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait Entry {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub struct FlushParam {
    pub collection: String,
    pub field: String,
}

pub struct SearchResult {
    pub detail: Box<dyn SearchResultDetail>
}

pub trait SearchResultDetail {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}
