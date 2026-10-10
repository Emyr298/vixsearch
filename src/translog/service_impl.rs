use std::{sync::{Arc, atomic::AtomicU64}, thread, time::Duration};

use dashmap::DashMap;

use crate::{collection::CollectionLifecycleEvent, document::OpSeqCommitEvent, errcode::{FATAL_ERROR, NOT_FOUND}, translog::{GetTransactionsParam, InsertPortParam, LoadStateResult, OpSeqStateDropEvent, Transaction, TranslogPort, TranslogService, TranslogStreamer, service_param_result::InsertParam, state::{OpSeqState, SegmentState}, streamer_impl::TranslogStreamerImpl}, utils::{handler::handler::Handler, observer::{observer::{Observer, ObserverGroupReader}, observer_group::ObserverGroup}, vixerr::Error, vixpool::Pool, vixscheduler::Scheduler}};

pub struct TranslogServiceImpl {
    adapter: Arc<dyn TranslogPort>,
    op_seq_state_by_collection_id: DashMap<String, Arc<OpSeqState>>,
    segment_state_by_segment_id: DashMap<String, Arc<SegmentState>>,
    commit_pool: Arc<dyn Pool>,
    op_seq_state_drop_observers: Arc<dyn ObserverGroupReader<OpSeqStateDropEvent>>,
}

impl TranslogServiceImpl {
    pub fn new(
        adapter: Arc<dyn TranslogPort>,
        commit_pool: Arc<dyn Pool>,
    ) -> (Arc<dyn TranslogService>, Arc<dyn Handler<CollectionLifecycleEvent>>, Arc<dyn Observer<OpSeqCommitEvent>>) {
        let (op_seq_state_drop_readers, op_seq_state_drop_writers) = ObserverGroup::new(Vec::<Arc<dyn Observer<OpSeqStateDropEvent>>>::new());

        let arc = Arc::new(TranslogServiceImpl {
            adapter,
            op_seq_state_by_collection_id: DashMap::new(),
            segment_state_by_segment_id: DashMap::new(),
            commit_pool,
            op_seq_state_drop_observers: op_seq_state_drop_readers,
        });

        op_seq_state_drop_writers.register(arc.clone());

        (arc.clone(), arc.clone(), arc)
    }

    fn load_or_create(&self, collection_id: &str) -> Result<(), Error> {
        let segment_id = uuid::Uuid::new_v4().to_string();
        let result = self.adapter.load_or_create(&collection_id, &segment_id)?;

        for segment in &result.segments {
            self.segment_state_by_segment_id.insert(segment.id.clone(), Arc::new(segment.segment_state()));
        }

        let state_arc = Arc::new(OpSeqState::new(&collection_id, &result.segment_ids(), result.committed_op_seq, self.op_seq_state_drop_observers.clone()));
        self.op_seq_state_by_collection_id.insert(collection_id.to_string(), state_arc);

        Ok(())
    }
}

impl TranslogService for TranslogServiceImpl {
    fn stream(&self, collection_id: &str) -> Result<Box<dyn TranslogStreamer>, Error> {
        let state = match self.op_seq_state_by_collection_id.get(collection_id) {
            Some(c) => c.clone(),
            None => return Error::code(FATAL_ERROR)
                .message(format!("Collection not found: {}", collection_id))
                .throw(),
        };

        let streamer = TranslogStreamerImpl::new(
            self.adapter.clone(),
            collection_id.to_string(),
            state.committed(),
        );

        Ok(streamer)
    }

    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        self.adapter.insert(param)
    }
}

impl Handler<CollectionLifecycleEvent> for TranslogServiceImpl {
    fn handle(&self, event: &CollectionLifecycleEvent) -> Result<(), Error> {
        match event {
            CollectionLifecycleEvent::Loaded(e) => {
                for collection_id in &e.collection_ids {
                    self.load_or_create(collection_id)?;
                }
                                
                Ok(())
            },

            CollectionLifecycleEvent::Created(e) => {
                self.load_or_create(&e.collection_id)
            },

            CollectionLifecycleEvent::Deleted(e) => {
                let state = match self.op_seq_state_by_collection_id.remove(&e.collection_id) {
                    Some((_, state)) => state,
                    None => return Ok(()),
                };

                for segment_id in &state.segment_ids {
                    self.segment_state_by_segment_id.remove(segment_id);
                }

                Ok(())
            },
        }
    }
}

impl Observer<OpSeqCommitEvent> for TranslogServiceImpl {
    fn observe(&self, event: &OpSeqCommitEvent) {
        let op_seq_state = match self.op_seq_state_by_collection_id.get(&event.collection_id) {
            Some(c) => c.clone(),
            None => {
                println!("Failed to op_seq_state_by_collection_id.get: collection id {} not found", event.collection_id);
                return;
            },
        };

        op_seq_state.set_pending_if_greater(event.op_seq);

        if !op_seq_state.is_commitable() {
            return;
        }

        let adapter = self.adapter.clone();
        let pool_result = self.commit_pool.submit(Box::new(move || {
            let session = match op_seq_state.try_begin_commit() {
                Some(s) => s,
                None => return,
            };

            if let Err(e) = adapter.commit(session.op_seq()) {
                eprintln!("Failed to commit op_seq {}: {}", session.op_seq(), e);
            }
        }));
        if let Err(e) = pool_result {
            eprintln!("Failed to submit commit task: {}", e);
        }
    }
}

// TODO: maybe we can make it work in another thread to not block the main thread -> for another services too
impl Observer<OpSeqStateDropEvent> for TranslogServiceImpl {
    fn observe(&self, event: &OpSeqStateDropEvent) {
        let collection_id = event.collection_id.clone();
        let adapter = self.adapter.clone();

        thread::spawn(move || {
            if let Err(e) = adapter.unload(&collection_id) {
                println!("Failed to release translog for collection {}: {}", collection_id, e);
            }
        });
    }
}

// TODO: segment drop event?
