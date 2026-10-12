use std::{collections::VecDeque, ops::Deref, process::Output, sync::{Arc, Mutex, MutexGuard, RwLock, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}}, time::{Duration, Instant, SystemTime}};

use arc_swap::ArcSwap;
use crossbeam::queue::SegQueue;
use dashmap::DashMap;

use crate::{translog::{InsertPortResult, LoadOrCreateResult, LoadOrCreateResultSegment, SegmentStateDropEvent}, utils::{active::active::{ActiveGuard, ActiveHandle}, observer::observer::ObserverGroupReader, vixerr::Error}};

pub struct CollectionState {
    collection_id: String,

    append_lock: Mutex<()>,

    active_segment_next_offset: AtomicU64,
    active_segment_byte_size: AtomicU64,
    segments: ArcSwap<Vec<Arc<SegmentState>>>, // TODO: reassess whether active handle is needed at all -> accessor can be from difference pod (replica)

    drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
}

impl CollectionState {
    pub fn new(
        collection_id: &str,
        load_or_create_result: LoadOrCreateResult,
        segment_drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
    ) -> Self {
        let mut seg = load_or_create_result.segments.into_iter()
            .map(|s| Arc::new(SegmentState{
                id: s.id,
                collection_id: collection_id.to_string(),
                minimum_op_seq: s.minimum_op_seq,
                drop_observers: segment_drop_observers.clone(),
            }))
            .collect::<Vec<Arc<SegmentState>>>();

        Self {
            collection_id: collection_id.to_string(),
            append_lock: Mutex::new(()),
            active_segment_next_offset: AtomicU64::new(load_or_create_result.active_segment_next_offset),
            active_segment_byte_size: AtomicU64::new(load_or_create_result.active_segment_byte_size),
            segments: ArcSwap::from_pointee(seg),
            drop_observers: segment_drop_observers
        }
    }

    pub fn is_active_segment(&self, id: &str) -> bool {
        let seg = self.segments.load();
        let active_seg = seg.last().unwrap();

        &active_seg.id == id
    }

    pub fn snapshot_segments(&self) -> Arc<Vec<Arc<SegmentState>>> {
        self.segments.load_full()
    }

    pub fn insert(&self, op_seq: u64, insert_fn: Box<dyn FnOnce(String, u64) -> Result<InsertPortResult, Error> + Send + 'static>) -> Result<(), Error> {
        let _lock_guard = self.append_lock.lock().unwrap();

        let segments = self.segments.load_full();
        let active_segment = segments.last().unwrap();

        let active_segment_next_offset = self.active_segment_next_offset.load(Ordering::Acquire);

        let result = insert_fn(active_segment.id.clone(), active_segment_next_offset)?;

        self.active_segment_byte_size.store(result.active_segment_next_offset, Ordering::Release);
        self.active_segment_byte_size.store(result.active_segment_byte_size, Ordering::Release);

        if let Some(added_segment) = result.added_segment {
            let mut updated_segments = (*segments).clone();

            updated_segments.push(Arc::new(SegmentState {
                id: added_segment.id,
                collection_id: self.collection_id.clone(),
                minimum_op_seq: added_segment.minimum_op_seq,
                drop_observers: self.drop_observers.clone(),
            }));

            self.segments.store(Arc::new(updated_segments));
        }
        Ok(())
    }
}

pub struct SegmentState {
    pub id: String,
    pub collection_id: String,
    pub minimum_op_seq: u64,

    drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
}

impl Drop for SegmentState {
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
