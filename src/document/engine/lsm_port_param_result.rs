use std::sync::Mutex;

use arc_swap::{ArcSwap, ArcSwapOption};
use fastbloom::BloomFilter;

use crate::document::engine::lsm_state::{CollectionBuffer, CollectionSegmentState, CollectionState, SegmentState};

pub struct GetMetadataPortResult {
    pub key_block_offsets: Vec<u64>,
    pub filter_hash_cnt: u32,
    pub filter_bits: Vec<u64>,
    pub smallest_key: Vec<u8>,
    pub biggest_key: Vec<u8>,
}

impl GetMetadataPortResult {
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

pub struct GetAllSegmentByCollectionIDPortResult {
    pub segments: Vec<GetAllSegmentByCollectionIDPortResultSegment>,
}

impl GetAllSegmentByCollectionIDPortResult {
    pub fn collection_state(&self, collection_id: &str) -> CollectionState {
        let segments: Vec<CollectionSegmentState> = self.segments.iter()
                .map(|s| s.collection_segment_state())
                .collect();

        CollectionState {
            id: collection_id.to_string(),
            segments: ArcSwap::from_pointee(segments),
            commit_lock: Mutex::new(()),
            buffer: ArcSwap::from_pointee(CollectionBuffer::new()),
            commit_buffer: ArcSwapOption::empty(),
        }
    }
}

pub struct GetAllSegmentByCollectionIDPortResultSegment {
    pub id: String,
    pub level: u32,
}

impl GetAllSegmentByCollectionIDPortResultSegment {
    pub fn collection_segment_state(&self) -> CollectionSegmentState {
        CollectionSegmentState {
            id: self.id.to_string(),
            level: self.level,
        }
    }
}
