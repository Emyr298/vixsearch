use std::{hint::spin_loop, sync::{Arc, Mutex, MutexGuard, atomic::{AtomicUsize, Ordering}}};

use arc_swap::{ArcSwap, ArcSwapOption};
use dashmap::{DashMap, iter::Iter, mapref::multiple::RefMulti};
use fastbloom::BloomFilter;
use rpds::{Vector, VectorSync};

use crate::{document::engine::{COMMIT_IN_PROGRESS, lsm_port_param_result::GetAllSegmentByCollectionIDPortResult, lsm_state::BufferState::Committing}, errcode::{self, FATAL_ERROR, SYSTEM_ERROR}, utils::vixerr::Error};

pub struct CollectionBuffer {
    byte_size: AtomicUsize,
    in_flight: AtomicUsize,
    map: DashMap<Vec<u8>, Vec<u8>>,
}

impl CollectionBuffer {
    fn new() -> Self {
        CollectionBuffer {
            byte_size: AtomicUsize::new(0),
            in_flight: AtomicUsize::new(0),
            map: DashMap::new(),
        }
    }

    pub fn sorted_key_values(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut key_values: Vec<_> = self.map.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        key_values.sort_by(|a, b| a.0.cmp(&b.0));
        key_values
    }

    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.map.get(key) {
            Some(value) => Some(value.clone()),
            None => None
        }
    }

    fn insert(&self, key: Vec<u8>, value: Vec<u8>) {
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

    fn add_in_flight(&self) {
        self.in_flight.fetch_add(1, Ordering::SeqCst);
    }

    fn sub_in_flight(&self) {
        self.in_flight.fetch_sub(1, Ordering::Release);
    }

    fn drain_in_flight(&self) {
        while self.in_flight.load(Ordering::Acquire) > 0 {
            spin_loop();
        }
    }

    fn byte_size(&self) -> usize {
        self.byte_size.load(Ordering::Relaxed)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

enum BufferState {
    Idle { buffer: Arc<CollectionBuffer> },
    Committing { buffer: Arc<CollectionBuffer>, commit_buffer: Arc<CollectionBuffer> }
}

pub struct CollectionState {
    pub id: String,
    commit_lock: Mutex<()>,
    buffer: ArcSwap<BufferState>,
    segments: ArcSwap<VectorSync<Arc<CollectionSegmentState>>>, // ascending by (level, creation)
}

impl CollectionState {
    pub fn from_get_all_segment_by_collection_id_port_result(id: &str, result: GetAllSegmentByCollectionIDPortResult) -> Self {
        let segments: VectorSync<Arc<CollectionSegmentState>> = result.segments.iter()
            .map(|s| Arc::new(CollectionSegmentState {
                id: s.id.clone(),
                level: s.level,
            }))
            .collect();

        Self {
            id: id.to_string(),
            commit_lock: Mutex::new(()),
            buffer: ArcSwap::from_pointee(BufferState::Idle {
                buffer: Arc::new(CollectionBuffer::new()),
            }),
            segments: ArcSwap::from_pointee(segments),
        }
    }

    pub fn insert_segment(&self, id: &str, level: u32) {
        self.segments.rcu(|s| {
            let mut segments = (**s).clone();
            let segment_arc = Arc::new(CollectionSegmentState {
                id: id.to_string(),
                level: level,
            });

            match segments.iter().position(|s| s.level > level) {
                Some(index) => segments.iter()
                    .take(index)
                    .cloned()
                    .chain(std::iter::once(segment_arc))
                    .chain(segments.iter().skip(index).cloned())
                    .collect(),
                None => segments.push_back(segment_arc),
            }
        });
    }

    pub fn value_from_buffer(&self, key: &[u8]) -> Option<Vec<u8>> {
        match &**self.buffer.load() {
            BufferState::Idle { buffer } => {
                buffer.get(key)
            },
            BufferState::Committing { buffer, commit_buffer } => {
                match buffer.get(key) {
                    Some(value) => Some(value),
                    None => commit_buffer.get(key)
                }
            },
        }
    }

    pub fn insert_buffer_value(&self, key: &[u8], value: &[u8]) {
        let buffer = loop {
            let candidate_state = self.buffer.load();
            let candidate_buffer = match &**candidate_state {
                BufferState::Idle { buffer } => buffer,
                BufferState::Committing { buffer, .. } => buffer,
            };

            candidate_buffer.add_in_flight();

            let active_state = self.buffer.load();
            let active_buffer = match &**active_state {
                BufferState::Idle { buffer } => buffer,
                BufferState::Committing { buffer, .. } => buffer,
            };

            // ensures after committed, we can't add in_flight anymore
            if Arc::ptr_eq(&candidate_buffer, &active_buffer) {
                break candidate_buffer.clone();
            }

            candidate_buffer.sub_in_flight();
        };

        buffer.insert(key.to_vec(), value.to_vec());
        buffer.sub_in_flight();
    }

    /// Returns a vector of all segment IDs in the collection, across all levels sorted from L0 to Ln.
    pub fn get_segment_ids(&self) -> Vec<String> {
        let segments = self.segments.load();
        segments.iter()
            .map(|s| s.id.clone())
            .collect()
    }

    pub fn is_flushable(&self, flush_byte_size_threshold: usize) -> bool {
        let cur = self.buffer.load();
        let byte_size = match &**cur {
            BufferState::Idle { buffer } => buffer.byte_size(),
            BufferState::Committing { .. } => return false,
        };

        byte_size < flush_byte_size_threshold
    }

    pub fn try_begin_commit<'a>(&'a self) -> Option<CommitGuard<'a>> {
        let lock_guard = match self.commit_lock.try_lock() {
            Ok(val) => val,
            Err(_) => return None,
        };

        let cur_state = self.buffer.load();
        let commit_buffer = match &**cur_state {
            BufferState::Idle { buffer } => buffer,
            BufferState::Committing { .. } => return None,
        };

        let next_state = Arc::new(BufferState::Committing {
            buffer: Arc::new(CollectionBuffer::new()),
            commit_buffer: commit_buffer.clone(),
        });

        self.buffer.store(next_state.clone());

        commit_buffer.drain_in_flight();
        Some(CommitGuard::new(self, lock_guard, commit_buffer.clone()))
    }
}

pub struct CollectionSegmentState {
    pub id: String,
    pub level: u32,
    // TODO: inflight when implementing compaction (segment can be erased)
}

pub struct CommitGuard<'a> {
    collection: &'a CollectionState,
    _lock_guard: MutexGuard<'a, ()>,
    pub buffer: Arc<CollectionBuffer>,
}

impl<'a> CommitGuard<'a> {
    fn new(collection: &'a CollectionState, lock_guard: MutexGuard<'a, ()>, commit_buffer: Arc<CollectionBuffer>) -> Self {
        Self {
            collection: collection,
            _lock_guard: lock_guard,
            buffer: commit_buffer,
        }
    }
}

impl<'a> Drop for CommitGuard<'a> {
    fn drop(&mut self) {
        let cur_state = self.collection.buffer.load();
        let next_state = match &**cur_state {
            BufferState::Idle { .. } => unreachable!("guard implies committing"),
            BufferState::Committing { buffer, .. } => Arc::new(BufferState::Idle {
                buffer: buffer.clone(),
            }),
        };
        self.collection.buffer.store(next_state);
    }
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
