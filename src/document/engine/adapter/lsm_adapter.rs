use std::sync::Arc;

use crate::{document::engine::{LSMPort, lsm_state::CollectionBuffer}, storage::BlockStorage, utils::vixerr::Error};

pub struct LSMAdapter {
    storage: Arc<dyn BlockStorage>,
}

impl LSMPort for LSMAdapter {
    fn get_values_from_block(&self, segment_id: &str, block_offset: u64) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error> {
        todo!()
    }

    fn flush_segment(&self, collection_buffer: Arc<CollectionBuffer>) -> Result<(), Error> {
        todo!()
    }
}