use std::{collections::VecDeque, ops::Deref, process::Output, sync::{Arc, Mutex, MutexGuard, RwLock, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}}};

use crossbeam::queue::SegQueue;
use dashmap::DashMap;

use crate::{translog::{LoadOrCreateResult, LoadOrCreateResultSegment, OpSeqStateDropEvent}, utils::{observer::observer::ObserverGroupReader, vixerr::Error}};

// move to common

struct Inner<T> {
    inner: T,
    is_deactivating: AtomicBool,
    access_count: AtomicUsize,
}

pub struct ActiveGuard<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Deref for ActiveGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.inner
    }
}

impl<T> Drop for ActiveGuard<T> {
    fn drop(&mut self) {
        self.inner.access_count.fetch_sub(1, Ordering::AcqRel);
    }
}

pub struct ActiveHandle<T> {
    inner: Arc<Inner<T>>,
}

impl<T> ActiveHandle<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(Inner {
                inner,
                is_deactivating: AtomicBool::new(false),
                access_count: AtomicUsize::new(0),
            }),
        }
    }

    pub fn get(&self) -> Option<ActiveGuard<T>> {
        if self.inner.is_deactivating.load(Ordering::Acquire) {
            return None;
        }

        self.inner.access_count.fetch_add(1, Ordering::AcqRel);

        if self.inner.is_deactivating.load(Ordering::Acquire) {
            self.inner.access_count.fetch_sub(1, Ordering::AcqRel);
            return None;
        }

        Some(ActiveGuard { inner: self.inner.clone() })
    }

    pub fn mark_for_deactivation(&self) {
        self.inner.is_deactivating.store(true, Ordering::Release);
    }

    pub fn is_deactivable(&self) -> bool {
        self.inner.is_deactivating.load(Ordering::Acquire)
            && self.inner.access_count.load(Ordering::Acquire) == 0
    }
}

// end

pub struct CollectionState {
    collection_id: String,

    // op seq
    last_committed_op_seq: AtomicU64,
    last_pending_op_seq: AtomicU64,
    commit_lock: Mutex<()>,

    // segments
    segments: RwLock<VecDeque<ActiveHandle<CollectionStateSegment>>>,

    // drop_observers: Arc<dyn ObserverGroupReader<OpSeqStateDropEvent>>,

    segment_byte_size_threshold: u64,
}

// TODO: purge unused segments -> also on segment purge event
pub struct CollectionStateSegment {
    pub id: String,
    pub byte_size: AtomicU64, // on load, should add one if last segment byte size reached threshold
    pub minimum_op_seq: u64,
}

impl CollectionState {
    pub fn new(
        collection_id: &str,
        load_or_create_result: LoadOrCreateResult,
        segment_byte_size_threshold: u64,
    ) -> Self {
        let no_active_segment = load_or_create_result.segments.last()
            .map_or(true, |s| s.byte_size >= segment_byte_size_threshold);

        let mut vec_segments = load_or_create_result.segments.into_iter()
            .map(|s| ActiveHandle::new(CollectionStateSegment{
                id: s.id,
                byte_size: AtomicU64::new(s.byte_size),
                minimum_op_seq: s.minimum_op_seq,
            }))
            .collect::<VecDeque<ActiveHandle<CollectionStateSegment>>>();

        if no_active_segment {
            vec_segments.push_back(ActiveHandle::new(CollectionStateSegment{
                id: uuid::Uuid::new_v4().to_string(),
                byte_size: AtomicU64::new(0),
                minimum_op_seq: load_or_create_result.last_pending_op_seq + 1,
            }));
        }

        Self {
            collection_id: collection_id.to_string(),
            last_committed_op_seq: AtomicU64::new(load_or_create_result.last_committed_op_seq),
            last_pending_op_seq: AtomicU64::new(load_or_create_result.last_pending_op_seq),
            commit_lock: Mutex::new(()),
            segments: RwLock::new(vec_segments),
            segment_byte_size_threshold: segment_byte_size_threshold,
        }
    }

    pub fn insert(&self, op_seq: u64, insert_fn: Box<dyn FnOnce(String) -> Result<u64, Error> + Send + 'static>) -> Result<(), Error> {
        let mut segments = self.segments.write().unwrap();
        let handle = segments.back().unwrap();
        let active_segment = handle.get().unwrap();

        let added_byte_size = insert_fn(active_segment.id.clone())?;

        let prev_byte_size = active_segment.byte_size.fetch_add(added_byte_size, Ordering::Release);

        let cur_byte_size = prev_byte_size + added_byte_size;
        if cur_byte_size >= self.segment_byte_size_threshold {
            let segment = CollectionStateSegment {
                id: uuid::Uuid::new_v4().to_string(),
                byte_size: AtomicU64::new(0),
                minimum_op_seq: op_seq + 1,
            };

            segments.push_back(ActiveHandle::new(segment));
        }

        Ok(())
    }

    pub fn segment_by_op_seq(&self, op_seq: u64) -> Option<ActiveGuard<CollectionStateSegment>> {
        let segments = self.segments.read().unwrap();

        for handle in segments.iter().rev() {
            let segment = match handle.get() {
                Some(active_segment) => active_segment,
                None => break,
            };

            if op_seq >= segment.minimum_op_seq {
                return Some(segment);
            }
        }

        None
    }

    pub fn last_committed_op_seq(&self) -> u64 {
        self.last_committed_op_seq.load(Ordering::Acquire)
    }

    pub fn set_last_pending_op_seq_if_greater(&self, value: u64) {
        self.last_pending_op_seq.fetch_max(value, Ordering::AcqRel);
    }

    pub fn try_commit(&self, commit_fn: Box<dyn FnOnce(u64) -> Result<(), Error> + Send + 'static>) -> Result<(), Error> {
        let _lock_guard = match self.commit_lock.lock() {
            Ok(l) => l,
            Err(_) => return Ok(()),
        };

        let last_pending_op_seq = self.last_pending_op_seq.load(Ordering::Acquire);
        let last_committed_op_seq = self.last_committed_op_seq.load(Ordering::Acquire);

        if last_pending_op_seq <= last_committed_op_seq {
            return Ok(());
        }

        commit_fn(last_pending_op_seq)?;

        self.last_committed_op_seq.fetch_max(last_pending_op_seq, Ordering::Release);

        Ok(())
    } 
}

// impl Drop for CollectionState {
//     fn drop(&mut self) {
//         let event = OpSeqStateDropEvent {
//             collection_id: self.collection_id.clone(),
//         };

//         for observer in self.drop_observers.list().iter() {
//             observer.observe(&event);
//         }
//     }
// }
