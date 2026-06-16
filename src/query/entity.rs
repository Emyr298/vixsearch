use std::any::Any;

use crate::shared::Value;

pub trait SearchQuery {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub struct EqualityQuery {
    pub value: Value,
}

impl SearchQuery for EqualityQuery {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
