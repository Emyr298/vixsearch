use std::{any::Any, collections::HashMap};

pub struct Query {
    pub collection: String,
    pub action: Action,
}

pub enum Action {
    Insert(InsertData),
}

pub struct InsertData {
    pub id: i64,
    pub payload: HashMap<String, Box<dyn Any>>,
}
