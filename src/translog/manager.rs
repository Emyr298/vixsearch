use crate::{error::Error, query};

// TODO: sharding
pub trait Manager {
    fn write_insert(&self, collection: String, operation: query::InsertData) -> Result<(), Error>;
}
