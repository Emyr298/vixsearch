use std::{sync::{Arc, Mutex}};

use arc_swap::{ArcSwap, ArcSwapOption};
use dashmap::DashMap;
use fastbloom::BloomFilter;

use crate::{document::engine::lsm_errors::FLUSH_ON_PROGRESS, errcode, utils::vixerr::Error};

pub struct CollectionState {
    pub id: String,
    pub levels: ArcSwap<Vec<LevelState>>,

    pub commit_lock: Mutex<()>,
    pub buffer: ArcSwap<DashMap<String, Vec<u8>>>,
    pub commit_buffer: ArcSwapOption<DashMap<String, Vec<u8>>>,
}

impl CollectionState {
    pub fn value_from_buffer(&self, key: &str) -> Option<Vec<u8>> {
        let buffer = self.buffer.load();

        if let Some(value) = buffer.get(key) {
            return Some(value.clone());
        }

        let commit_buffer = self.commit_buffer.load();
        if let Some(cb) = commit_buffer.as_ref() {
            if let Some(value) = cb.get(key) {
                return Some(value.clone());
            }
        }

        None
    }

    /// Returns a vector of all segment IDs in the collection, across all levels sorted from L0 to Ln.
    pub fn get_segment_ids(&self) -> Vec<String> {
        let levels = self.levels.load();
        levels.iter()
            .flat_map(|level| level.segment_ids.iter().cloned())
            .collect()
    }

    pub fn flush_buffer(&self) -> Result<(), Error> {
        let _guard = match self.commit_lock.try_lock() {
            Ok(val) => val,
            Err(_) => return Error::code(FLUSH_ON_PROGRESS)
                .message("flush is on progress")
                .throw(),
        };

        let clean_buffer = DashMap::new();
        let old_buffer = self.buffer.swap(Arc::new(clean_buffer)); 
        let old_commit_buffer = self.commit_buffer.swap(Some(old_buffer));
        if old_commit_buffer.is_some() {
            return Error::code(errcode::FATAL_ERROR)
                .message("commit buffer is not empty on commit")
                .throw()
        }

        // do something with adapter
        

        self.commit_buffer.swap(None);
        Ok(())
    }
}

pub struct LevelState {
    pub segment_ids: Vec<String>,
}

pub struct SegmentState {
    pub id: String,
    pub collection_id: String,

    pub smallest_key: String,
    pub biggest_key: String,
    pub key_filter: BloomFilter,
    pub key_block_offsets: Vec<u64>,
}

impl SegmentState {
    pub fn may_contain_key(&self, key: &str) -> bool {
        if key < self.smallest_key.as_str() || key > self.biggest_key.as_str() {
            return false;
        }

        self.key_filter.contains(&key)
    } 
}
