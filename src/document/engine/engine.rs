use crate::{document::engine::param_result::InsertParam, utils::vixerr::Error};

pub trait Engine: Send + Sync {
    fn get_by_key(&self, collection_id: &str, key: &str) -> Result<Vec<u8>, Error>;
    fn insert(&self, collection_id: &str, param: InsertParam) -> Result<(), Error>;
    fn batch_insert(&self, collection_id: &str, param: Vec<InsertParam>) -> Result<(), Error>;
    fn flush(&self, collection_id: &str) -> Result<(), Error>;
}

pub trait Port: Send + Sync {
    fn get_values_from_block(&self, segment_id: &str, block_offset: u64) -> Result<Vec<(String, Vec<u8>)>, Error>;
    fn flush_segment(&self) -> Result<(), Error>;
}
