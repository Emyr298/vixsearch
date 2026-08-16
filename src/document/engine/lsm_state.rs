use std::{hint::spin_loop, sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}}};

use arc_swap::{ArcSwap, ArcSwapOption};
use dashmap::DashMap;
use fastbloom::BloomFilter;

use crate::{errcode, utils::vixerr::Error};

pub struct CollectionBuffer {
    pub byte_size: AtomicUsize,
    pub in_flight: AtomicUsize,
    pub map: DashMap<Vec<u8>, Vec<u8>>,
}

impl CollectionBuffer {
    pub fn new() -> Self {
        CollectionBuffer {
            byte_size: AtomicUsize::new(0),
            in_flight: AtomicUsize::new(0),
            map: DashMap::new(),
        }
    }

    pub fn insert(&self, key: Vec<u8>, value: Vec<u8>) {
        let key_size = key.len();
        let cur_value_size = value.len();

        let cur_approx_size = key_size + cur_value_size;
        let old_approx_size = match self.map.insert(key, value) {
            Some(old_value) => key_size + old_value.len(),
            None => 0,
        };

        if cur_approx_size > old_approx_size {
            self.byte_size.fetch_add(cur_approx_size - old_approx_size, Ordering::Relaxed);
        } else {
            self.byte_size.fetch_sub(old_approx_size - cur_approx_size, Ordering::Relaxed);
        }
    }

    pub fn add_in_flight(&self) {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
    }

    pub fn sub_in_flight(&self) {
        self.in_flight.fetch_sub(1, Ordering::Release);
    }
}

pub struct CollectionState {
    pub id: String,
    pub segments: ArcSwap<Vec<CollectionSegmentState>>,

    pub commit_lock: Mutex<()>,
    pub buffer: ArcSwap<CollectionBuffer>,
    pub commit_buffer: ArcSwapOption<CollectionBuffer>,
}

impl CollectionState {
    pub fn value_from_buffer(&self, key: &[u8]) -> Option<Vec<u8>> {
        let buffer = self.buffer.load();

        if let Some(value) = buffer.map.get(key) {
            return Some(value.clone());
        }

        let commit_buffer = self.commit_buffer.load();
        if let Some(cb) = commit_buffer.as_ref() {
            if let Some(value) = cb.map.get(key) {
                return Some(value.clone());
            }
        }

        None
    }

    /// Returns a vector of all segment IDs in the collection, across all levels sorted from L0 to Ln.
    pub fn get_segment_ids(&self) -> Vec<String> {
        let levels = self.segment_levels.load();
        levels.iter()
            .flat_map(|level| level.segment_ids.iter().cloned())
            .collect()
    }

    pub fn start_flush(&self) -> Result<(), Error> {
        let clean_buffer = CollectionBuffer::new();
        let old_buffer = self.buffer.swap(Arc::new(clean_buffer)); 
        let old_commit_buffer = self.commit_buffer.swap(Some(Arc::clone(&old_buffer)));
        if old_commit_buffer.is_some() {
            return Error::code(errcode::FATAL_ERROR)
                .message("commit buffer is not empty on commit")
                .throw()
        }

        while old_buffer.in_flight.load(Ordering::Acquire) > 0 {
            spin_loop();
        }

        Ok(())
    }

    pub fn end_flush(&self) {
        self.commit_buffer.swap(None);
    }
}

pub struct CollectionSegmentState {
    pub id: String,
    pub level: u32,
    // TODO: inflight
}

pub struct SegmentState {
    pub id: String,
    pub collection_id: String,

    pub smallest_key: Vec<u8>,
    pub biggest_key: Vec<u8>,
    pub key_filter: BloomFilter,
    pub key_block_offsets: Vec<u64>,
}

impl SegmentState {
    pub fn may_contain_key(&self, key: &[u8]) -> bool {
        if key < self.smallest_key.as_slice() || key > self.biggest_key.as_slice() {
            return false;
        }

        self.key_filter.contains(&key)
    }
}
