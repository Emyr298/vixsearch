use std::sync::Arc;

use crate::{document::engine::{lsm_entity::SegmentMetadata, lsm_port_param_result::GetAllSegmentByCollectionIDPortResult}, utils::vixerr::Error};

pub trait DocumentEngine: Send + Sync {
    fn get_by_key(&self, collection_id: &str, key: &[u8]) -> Result<Vec<u8>, Error>;
    fn insert(&self, collection_id: &str, key: &[u8], value: &[u8]) -> Result<(), Error>;
}

pub trait DocumentEngineLoader: Send + Sync {
    fn load(&self, collection_ids: &[String]) -> Result<(), Error>;
}

pub trait LSMDocumentPort: Send + Sync {
    fn get_all_segment_by_collection_id(&self, collection_id: &str) -> Result<GetAllSegmentByCollectionIDPortResult, Error>;
    fn get_metadata(&self, collection_id: &str, segment_id: &str) -> Result<SegmentMetadata, Error>;
    fn get_values_from_block(&self, collection_id: &str, segment_id: &str, block_offset: u64) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error>;
    fn flush_segment(&self, collection_id: &str, segment_id: &str, sorted_key_values: Vec<(Vec<u8>, Vec<u8>)>) -> Result<SegmentMetadata, Error>;
}
