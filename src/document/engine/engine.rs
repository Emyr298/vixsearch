use std::sync::Arc;

use crate::{document::engine::lsm_state::CollectionBuffer, utils::vixerr::Error};

pub trait Engine: Send + Sync {
    fn get_by_key(&self, collection_id: &str, key: &[u8]) -> Result<Vec<u8>, Error>;
    fn insert(&self, collection_id: &str, key: &[u8], value: &[u8]) -> Result<(), Error>;
}

pub trait LSMPort: Send + Sync {
    fn get_values_from_block(&self, segment_id: &str, block_offset: u64) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error>;
    fn flush_segment(&self, collection_buffer: Arc<CollectionBuffer>) -> Result<(), Error>;
}
