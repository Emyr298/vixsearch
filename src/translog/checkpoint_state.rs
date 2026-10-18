use std::{sync::{Mutex, atomic::{AtomicU64, Ordering}}, time::{Duration, Instant}};

use crate::utils::vixerr::Error;

pub struct CollectionState {
    collection_id: String,
    last_checkpoint_time: Mutex<Instant>,
    checkpoint_interval: Duration,

    checkpoint_op_seq: AtomicU64,
    pending_op_seq: AtomicU64,
}

impl CollectionState {
    pub fn new(
        collection_id: &str,
        checkpoint_interval: Duration,
        checkpoint_op_seq: u64,
    ) -> Self {
        Self {
            collection_id: collection_id.to_string(),
            last_checkpoint_time: Mutex::new(Instant::now()),
            checkpoint_interval,
            checkpoint_op_seq: AtomicU64::new(checkpoint_op_seq),
            pending_op_seq: AtomicU64::new(checkpoint_op_seq),
        }
    }

    pub fn checkpoint_op_seq(&self) -> u64 {
        self.checkpoint_op_seq.load(Ordering::Acquire)
    }

    pub fn update_pending(&self, op_seq: u64) {
        self.pending_op_seq.fetch_max(op_seq, Ordering::AcqRel);
    }

    pub fn try_submit_checkpoint(&self, submit_fn: Box<dyn FnOnce(u64) -> Result<(), Error> + Send + 'static>) -> Result<(), Error> {
        let mut last_checkpoint_time = match self.last_checkpoint_time.lock() {
            Ok(l) => l,
            Err(_) => return Ok(()),
        };

        if last_checkpoint_time.elapsed() < self.checkpoint_interval {
            return Ok(());
        }

        let checkpoint_op_seq = self.checkpoint_op_seq.load(Ordering::Acquire);
        let pending_op_seq = self.pending_op_seq.load(Ordering::Acquire);

        if pending_op_seq <= checkpoint_op_seq {
            return Ok(());
        }

        submit_fn(pending_op_seq)?;

        self.checkpoint_op_seq.fetch_max(pending_op_seq, Ordering::AcqRel);
        *last_checkpoint_time = Instant::now();
        Ok(())
    } 
}

