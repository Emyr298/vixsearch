use std::any::Any;

use crate::{index::IndexType, query, shared::{Document, ValueType}};

pub struct CreateParam<'a> {
    pub collection: &'a str,
    pub fields: Vec<CreateParamField<'a>>,
}

pub struct CreateParamField<'a> {
    pub name: &'a str,
    pub index_type: IndexType,
}

// impl Into<TypeCreateParam> for CreateParam {
//     fn into(self) -> TypeCreateParam {
//         TypeCreateParam { collection: self.collection, field: self.field }
//     }
// }

pub struct TypeCreateParam {
    pub collection: String,
    pub field: String,
}

pub struct DeleteParam<'a> {
    pub collection: &'a str,
    pub field: &'a str,
}

pub struct GetParam<'a> {
    pub collection: &'a str,
    pub id: &'a str,
}

pub struct SearchParam {
    pub collection: String,
    pub field: String,
    pub query: Box<dyn query::SearchQuery>,
}

pub struct InsertParam<'a> {
    pub collection: &'a str,
    pub document: &'a Document,
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
