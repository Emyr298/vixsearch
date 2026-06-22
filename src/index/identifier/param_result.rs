use crate::{shared::Document};

pub struct CreateParam<'a> {
    pub collection: &'a str,
}

pub struct GetParam<'a> {
    pub collection: &'a str,
    pub id: &'a str,
}

pub struct InsertParam<'a> {
    pub collection: &'a str,
    pub id: &'a str,
    pub document: &'a Document,
}

pub struct DeleteParam<'a> {
    pub collection: &'a str,
}
