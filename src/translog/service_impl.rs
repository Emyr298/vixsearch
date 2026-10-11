use std::{sync::{Arc, atomic::AtomicU64}, thread, time::Duration};

use dashmap::DashMap;

use crate::{collection::CollectionLifecycleEvent, document::OpSeqCommitEvent, errcode::{FATAL_ERROR, NOT_FOUND}, translog::{GetTransactionsParam, InsertPortParam, SegmentStateDropEvent, Transaction, TranslogPort, TranslogService, TranslogStreamer, service_param_result::InsertParam, state::{CollectionState}, streamer_impl::TranslogStreamerImpl}, utils::{handler::handler::Handler, observer::{observer::{Observer, ObserverGroupReader}, observer_group::ObserverGroup}, vixerr::Error, vixpool::Pool, vixscheduler::Scheduler}};

pub struct TranslogServiceImpl {
    adapter: Arc<dyn TranslogPort>,
    state_by_collection_id: DashMap<String, Arc<CollectionState>>,
    commit_pool: Arc<dyn Pool>,
    segment_byte_size_threshold: u64,

    // observers
    segment_state_drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
}

impl TranslogServiceImpl {
    pub fn new(
        adapter: Arc<dyn TranslogPort>,
        commit_pool: Arc<dyn Pool>,
        segment_byte_size_threshold: u64,
    ) -> (Arc<dyn TranslogService>, Arc<dyn Handler<CollectionLifecycleEvent>>, Arc<dyn Observer<SegmentStateDropEvent>>) {

        let (drop_readers, drop_writers) = ObserverGroup::new(Vec::<Arc<dyn Observer<SegmentStateDropEvent>>>::new());

        let arc = Arc::new(TranslogServiceImpl {
            adapter,
            state_by_collection_id: DashMap::new(),
            commit_pool,
            segment_byte_size_threshold,
            segment_state_drop_observers: drop_readers,
        });

        drop_writers.register(arc.clone());

        (arc.clone(), arc.clone(), arc)
    }
}

impl TranslogService for TranslogServiceImpl {
    fn stream_uncommitted_transactions(&self, collection_id: &str) -> Result<Box<dyn TranslogStreamer>, Error> {
        let state = match self.state_by_collection_id.get(collection_id) {
            Some(c) => c.clone(),
            None => return Error::code(FATAL_ERROR)
                .message(format!("Collection not found: {}", collection_id))
                .throw(),
        };

        let streamer = TranslogStreamerImpl::new(
            self.adapter.clone(),
            state.clone(),
            collection_id.to_string(),
            state.last_committed_op_seq(),
        );

        Ok(streamer)
    }

    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        let state = match self.state_by_collection_id.get(&param.collection_id) {
            Some(c) => c.clone(),
            None => return Error::code(FATAL_ERROR)
                .message(format!("Collection not found: {}", param.collection_id))
                .throw(),
        };

        let adapter = self.adapter.clone();
        state.insert(param.op_seq, Box::new(move |active_segment_id| {
            let result = adapter.insert(param.insert_port_param(&active_segment_id))?;
            Ok(result.byte_size)
        }))
    }
}

impl Handler<CollectionLifecycleEvent> for TranslogServiceImpl {
    fn handle(&self, event: &CollectionLifecycleEvent) -> Result<(), Error> {
        match event {
            CollectionLifecycleEvent::Loaded(e) => {
                for collection_id in &e.collection_ids {
                    let result = self.adapter.load_or_create(collection_id)?;

                    self.state_by_collection_id.insert(
                        collection_id.clone(),
                        Arc::new(CollectionState::new(
                            collection_id,
                            result,
                            self.segment_byte_size_threshold,
                            self.segment_state_drop_observers.clone(),
                        ))
                    );
                }
            },
            CollectionLifecycleEvent::Created(e) => {
                let result = self.adapter.load_or_create(&e.collection_id)?;

                self.state_by_collection_id.insert(
                    e.collection_id.clone(),
                    Arc::new(CollectionState::new(
                        &e.collection_id,
                        result,
                        self.segment_byte_size_threshold,
                        self.segment_state_drop_observers.clone(),
                    ))
                );
            },
            CollectionLifecycleEvent::Deleted(e) => {
                self.state_by_collection_id.remove(&e.collection_id);
            },
        }

        Ok(())
    }
}

impl Observer<OpSeqCommitEvent> for TranslogServiceImpl {
    fn observe(&self, event: &OpSeqCommitEvent) {
        let state = match self.state_by_collection_id.get(&event.collection_id) {
            Some(c) => c.clone(),
            None => {
                println!("Failed to op_seq_state_by_collection_id.get: collection id {} not found", event.collection_id);
                return;
            },
        };

        state.set_last_pending_op_seq_if_greater(event.op_seq);

        let event = event.clone();
        let adapter = self.adapter.clone();

        if let Err(e) = self.commit_pool.submit(Box::new(move || {
            if let Err(e) = state.try_commit(Box::new(move |last_pending_op_seq| {
                adapter.commit_op_seq(last_pending_op_seq)
            })) {
                eprintln!("Failed to commit op_seq {}: {}", event.op_seq, e);
            }
        })) {
            eprintln!("Failed to submit commit task: {}", e);
        }
    }
}

// TODO: please for another services, make it thread::spawn or submit job (without block) so it won't block the publisher's thread
impl Observer<SegmentStateDropEvent> for TranslogServiceImpl {
    fn observe(&self, event: &SegmentStateDropEvent) {
        let event = event.clone();
        let adapter = self.adapter.clone();

        thread::spawn(move || {
            if let Err(e) = adapter.unload(&event.collection_id, &event.segment_id) {
                println!("Failed to release translog for collection {}: {}", &event.collection_id, e);
            }
        });
    }
}
