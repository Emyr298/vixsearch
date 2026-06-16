use std::any::Any;

use crate::{index, shared::Document};

pub fn document_result(document: Document) -> index::SearchResult {
    index::SearchResult {
        detail: Box::new(document),
    }
}

impl index::SearchResultDetail for Document {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl index::Entry for Document {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl index::Lookup for String {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
