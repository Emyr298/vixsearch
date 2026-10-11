use std::{collections::VecDeque, ops::Deref, process::Output, sync::{Arc, Mutex, MutexGuard, RwLock, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}}};

use crossbeam::queue::SegQueue;
use dashmap::DashMap;

use crate::{translog::{LoadOrCreateResult, LoadOrCreateResultSegment, SegmentStateDropEvent}, utils::{active::active::{ActiveGuard, ActiveHandle}, observer::observer::ObserverGroupReader, vixerr::Error}};

pub struct CollectionState {
    collection_id: String,

    // op seq
    last_committed_op_seq: AtomicU64,
    last_pending_op_seq: AtomicU64,
    commit_lock: Mutex<()>,

    // segments
    segments: RwLock<VecDeque<ActiveHandle<CollectionStateSegment>>>,
    segment_byte_size_threshold: u64,
    segment_drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
}

impl CollectionState {
    pub fn new(
        collection_id: &str,
        load_or_create_result: LoadOrCreateResult,
        segment_byte_size_threshold: u64,
        segment_drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
    ) -> Self {
        let no_active_segment = load_or_create_result.segments.last()
            .map_or(true, |s| s.byte_size >= segment_byte_size_threshold);

        let mut vec_segments = load_or_create_result.segments.into_iter()
            .map(|s| ActiveHandle::new(CollectionStateSegment{
                id: s.id,
                collection_id: collection_id.to_string(),
                byte_size: AtomicU64::new(s.byte_size),
                minimum_op_seq: s.minimum_op_seq,
                drop_observers: segment_drop_observers.clone(),
            }))
            .collect::<VecDeque<ActiveHandle<CollectionStateSegment>>>();

        if no_active_segment {
            vec_segments.push_back(ActiveHandle::new(CollectionStateSegment{
                id: uuid::Uuid::new_v4().to_string(),
                collection_id: collection_id.to_string(),
                byte_size: AtomicU64::new(0),
                minimum_op_seq: load_or_create_result.last_pending_op_seq + 1,
                drop_observers: segment_drop_observers.clone(),
            }));
        }

        Self {
            collection_id: collection_id.to_string(),
            last_committed_op_seq: AtomicU64::new(load_or_create_result.last_committed_op_seq),
            last_pending_op_seq: AtomicU64::new(load_or_create_result.last_pending_op_seq),
            commit_lock: Mutex::new(()),
            segments: RwLock::new(vec_segments),
            segment_byte_size_threshold: segment_byte_size_threshold,
            segment_drop_observers: segment_drop_observers,
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
                collection_id: self.collection_id.clone(),
                byte_size: AtomicU64::new(0),
                minimum_op_seq: op_seq + 1,
                drop_observers: self.segment_drop_observers.clone(),
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

pub struct CollectionStateSegment {
    pub id: String,
    pub collection_id: String,
    pub minimum_op_seq: u64,

    byte_size: AtomicU64,
    drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
}

impl Drop for CollectionStateSegment {
    fn drop(&mut self) {
        let event = SegmentStateDropEvent {
            collection_id: self.collection_id.clone(),
            segment_id: self.id.clone(),
        };

        for observer in self.drop_observers.list().iter() {
            observer.observe(&event);
        }
    }
}
