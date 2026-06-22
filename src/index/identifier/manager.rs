use crate::{index::identifier::{DeleteParam, param_result::{CreateParam, InsertParam, GetParam}}, shared::Document, utils::vixerr::Error};

pub trait Manager: Send + Sync {
    fn create(&self, param: CreateParam) -> Result<(), Error>;
    fn get(&self, param: GetParam) -> Result<Document, Error>;
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
    fn delete(&self, param: DeleteParam) -> Result<(), Error>;
}
