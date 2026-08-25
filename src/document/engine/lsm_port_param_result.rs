use std::sync::Mutex;

use arc_swap::{ArcSwap, ArcSwapOption};
use fastbloom::BloomFilter;

use crate::document::engine::lsm_state::{CollectionBuffer, CollectionSegmentState, CollectionState, SegmentState};

pub struct GetAllSegmentByCollectionIDPortResult {
    pub segments: Vec<GetAllSegmentByCollectionIDPortResultSegment>,
}

pub struct GetAllSegmentByCollectionIDPortResultSegment {
    pub id: String,
    pub level: u32,
}
