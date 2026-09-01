use crate::{document::Document, query::service_param_result::InsertParam, utils::vixerr::Error};

pub trait QueryService {
    fn get_by_id(&self, collection_id: &str, id: &str) -> Result<Document, Error>;
    fn insert(&self, param: InsertParam) -> Result<Document, Error>;
}
