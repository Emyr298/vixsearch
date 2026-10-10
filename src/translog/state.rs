use std::sync::{Arc, Mutex, MutexGuard, atomic::{AtomicU64, Ordering}};

use crate::{translog::OpSeqStateDropEvent, utils::observer::observer::ObserverGroupReader};

pub struct OpSeqState {
    lock: Mutex<()>,
    collection_id: String,
    committed: AtomicU64,
    pending: AtomicU64,
    pub segment_ids: Vec<String>,
    drop_observers: Arc<dyn ObserverGroupReader<OpSeqStateDropEvent>>,
}

impl OpSeqState {
    pub fn new(collection_id: &str, segment_ids: &[String], op_seq: u64, drop_observers: Arc<dyn ObserverGroupReader<OpSeqStateDropEvent>>) -> Self {
        Self {
            lock: Mutex::new(()),
            collection_id: collection_id.to_string(),
            committed: AtomicU64::new(op_seq),
            pending: AtomicU64::new(op_seq),
            segment_ids: segment_ids.to_vec(),
            drop_observers,
        }
    }

    pub fn active_segment_id(&self) -> String {
        self.segment_ids.last()
            .unwrap()
            .clone()
    }

    pub fn committed(&self) -> u64 {
        self.committed.load(Ordering::Acquire)
    }

    pub fn set_pending_if_greater(&self, op_seq: u64) {
        self.pending.fetch_max(op_seq, Ordering::Release);
    }

    pub fn is_commitable(&self) -> bool {
        let pending_op_seq = self.pending.load(Ordering::Acquire);
        let committed_op_seq = self.committed.load(Ordering::Acquire);

        pending_op_seq > committed_op_seq
    }

    pub fn try_begin_commit<'a>(&'a self) -> Option<CommitSession<'a>> {
        let lock_guard = match self.lock.try_lock() {
            Ok(val) => val,
            Err(_) => return None,
        };

        let pending_op_seq = self.pending.load(Ordering::Acquire);
        let committed_op_seq = self.committed.load(Ordering::Acquire);

        if pending_op_seq == committed_op_seq {
            return None
        }

        Some(CommitSession::new(self, lock_guard, pending_op_seq))
    }
}

impl Drop for OpSeqState {
    fn drop(&mut self) {
        let event = OpSeqStateDropEvent {
            collection_id: self.collection_id.clone(),
        };

        for observer in self.drop_observers.list().iter() {
            observer.observe(&event);
        }
    }
}

pub struct CommitSession<'a> {
    state: &'a OpSeqState,
    _lock_guard: MutexGuard<'a, ()>,
    op_seq: u64,
    committed: bool,
}

impl<'a> CommitSession<'a> {
    pub fn new(state: &'a OpSeqState, lock_guard: MutexGuard<'a, ()>, op_seq: u64) -> Self {
        Self {
            state,
            _lock_guard: lock_guard,
            op_seq,
            committed: false,
        }
    }

    pub fn op_seq(&self) -> u64 {
        self.op_seq
    }

    pub fn finalize(mut self) {
        self.finalize_internal();
    }

    fn finalize_internal(&mut self) {
        self.state.committed.fetch_max(self.op_seq, Ordering::Release);
        self.committed = true;
    }
}

impl<'a> Drop for CommitSession<'a> {
    fn drop(&mut self) {
        if !self.committed {
            self.finalize_internal();
        }
    }
}

pub struct SegmentState {
    pub id: String,
    pub minimum_op_seq: u64,
    // TODO: drop_observers
}
