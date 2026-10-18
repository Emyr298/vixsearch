use std::{sync::{Arc, atomic::AtomicU64}, thread, time::Duration};

use dashmap::DashMap;

use crate::{collection::CollectionLifecycleEvent, document::OpSeqCommitEvent, errcode::{FATAL_ERROR, NOT_FOUND}, translog::{GetTransactionsParam, InsertPortParam, SegmentStateDropEvent, StreamTrxLogsParam, Transaction, TranslogPort, TranslogService, TranslogStreamer, service_param_result::InsertParam, state::CollectionState, streamer_impl::TranslogStreamerImpl}, utils::{handler::handler::Handler, observer::{observer::{Observer, ObserverGroupReader}, observer_group::ObserverGroup}, vixerr::Error, vixpool::Pool, vixscheduler::Scheduler}};

pub struct TranslogServiceImpl {
    adapter: Arc<dyn TranslogPort>,
    state_by_collection_id: DashMap<String, Arc<CollectionState>>,
    segment_state_drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
    stream_insert_wait_timeout: Duration,
}

impl TranslogServiceImpl {
    pub fn new(
        adapter: Arc<dyn TranslogPort>,
        commit_pool: Arc<dyn Pool>,
        commit_interval: Duration,
        stream_insert_wait_timeout: Duration,
    ) -> (Arc<dyn TranslogService>, Arc<dyn Handler<CollectionLifecycleEvent>>, Arc<dyn Observer<SegmentStateDropEvent>>) {

        let (drop_readers, drop_writers) = ObserverGroup::new(Vec::<Arc<dyn Observer<SegmentStateDropEvent>>>::new());

        let arc = Arc::new(TranslogServiceImpl {
            adapter,
            state_by_collection_id: DashMap::new(),
            segment_state_drop_observers: drop_readers,
            stream_insert_wait_timeout,
        });

        drop_writers.register(arc.clone());

        (arc.clone(), arc.clone(), arc)
    }
}

impl TranslogService for TranslogServiceImpl {
    fn stream_trx_logs(&self, param: StreamTrxLogsParam) -> Result<Box<dyn TranslogStreamer>, Error> {
        let state = match self.state_by_collection_id.get(&param.collection_id) {
            Some(c) => c.clone(),
            None => return Error::code(FATAL_ERROR)
                .message(format!("Collection not found: {}", &param.collection_id))
                .throw(),
        };

        Ok(TranslogStreamerImpl::new(
            &param.collection_id,
            self.adapter.clone(),
            state.clone(),
            param.start_op_seq,
            param.is_continuous,
            param.max_batch,
            param.max_outstanding,
            self.stream_insert_wait_timeout,
        ))
    }

    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        let state = match self.state_by_collection_id.get(&param.collection_id) {
            Some(c) => c.clone(),
            None => return Error::code(FATAL_ERROR)
                .message(format!("Collection not found: {}", param.collection_id))
                .throw(),
        };

        let adapter = self.adapter.clone();
        state.insert(param.op_seq, Box::new(move |active_segment_id, active_segment_byte_size, active_segment_next_offset| {
            Ok(adapter.insert(param.insert_port_param(&active_segment_id, active_segment_byte_size, active_segment_next_offset))?)
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

// TODO: please for another services, make it thread::spawn or submit job (without block) so it won't block the publisher's thread
impl Observer<SegmentStateDropEvent> for TranslogServiceImpl {
    fn observe(&self, event: &SegmentStateDropEvent) {
        let event = event.clone();
        let adapter = self.adapter.clone();

        thread::spawn(move || {
            adapter.unload(&event.collection_id, &event.segment_id);
        });
    }
}
