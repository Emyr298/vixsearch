use fastbloom::BloomFilter;

use crate::document::engine::lsm_state::SegmentState;

pub struct SegmentMetadata {
    pub key_block_offsets: Vec<u64>,
    pub filter_hash_cnt: u32,
    pub filter_bits: Vec<u64>,
    pub smallest_key: Vec<u8>,
    pub biggest_key: Vec<u8>,
}

impl SegmentMetadata {
    pub fn segment_state(self, segment_id: &str, collection_id: &str) -> SegmentState {
        let key_filter = BloomFilter::from_vec(self.filter_bits).hashes(self.filter_hash_cnt);
        SegmentState {
            id: segment_id.to_string(),
            collection_id: collection_id.to_string(),
            smallest_key: self.smallest_key,
            biggest_key: self.biggest_key,
            key_filter: key_filter,
            key_block_offsets: self.key_block_offsets,
        }
    }
}
