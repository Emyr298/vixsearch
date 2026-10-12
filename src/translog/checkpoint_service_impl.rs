use std::{sync::Arc, time::Duration};

use dashmap::DashMap;

use crate::{collection::CollectionLifecycleEvent, document::OpSeqCommitEvent, errcode::FATAL_ERROR, translog::{CheckpointPort, CheckpointService, checkpoint_state::CollectionState}, utils::{handler::handler::Handler, observer::observer::Observer, vixerr::Error, vixpool::Pool}};

pub struct CheckpointServiceImpl {
    adapter: Arc<dyn CheckpointPort>,
    state_by_collection_id: DashMap<String, Arc<CollectionState>>,
    checkpoint_pool: Arc<dyn Pool>,
    checkpoint_interval: Duration,
}

impl CheckpointService for CheckpointServiceImpl {
    fn get_checkpoint_op_seq(&self, collection_id: &str) -> Result<u64, Error> {
        let state = match self.state_by_collection_id.get(collection_id) {
            Some(c) => c.clone(),
            None => return Error::code(FATAL_ERROR)
                .message(format!("Collection not found: {}", collection_id))
                .throw(),
        };
        Ok(state.checkpoint_op_seq())
    }
}

impl Observer<OpSeqCommitEvent> for CheckpointServiceImpl {
    fn observe(&self, event: &OpSeqCommitEvent) {
        let state = match self.state_by_collection_id.get(&event.collection_id) {
            Some(c) => c.clone(),
            None => {
                println!("Failed to state_by_collection_id.get: collection id {} not found", event.collection_id);
                return;
            },
        };

        state.update_pending(event.op_seq);
        self.should_submit_checkpoint(event.op_seq, state);
    }
}

impl CheckpointServiceImpl {
    fn should_submit_checkpoint(&self, op_seq: u64, state: Arc<CollectionState>) {
        let adapter = self.adapter.clone();

        if let Err(e) = self.checkpoint_pool.submit(Box::new(move || {
            if let Err(e) = state.try_submit_checkpoint(Box::new(move |pending_op_seq| {
                adapter.submit_checkpoint(pending_op_seq)
            })) {
                eprintln!("Failed to commit op_seq {}: {}", op_seq, e);
            }
        })) {
            eprintln!("Failed to submit commit task: {}", e);
        }
    }
}

impl Handler<CollectionLifecycleEvent> for CheckpointServiceImpl {
    fn handle(&self, event: &CollectionLifecycleEvent) -> Result<(), Error> {
        match event {
            CollectionLifecycleEvent::Loaded(e) => {
                for collection_id in &e.collection_ids {
                    let op_seq = self.adapter.get_checkpoint_op_seq(collection_id)?;

                    self.state_by_collection_id.insert(
                        collection_id.clone(),
                        Arc::new(CollectionState::new(
                            collection_id,
                            self.checkpoint_interval,
                            op_seq,
                        ))
                    );
                }
            },
            CollectionLifecycleEvent::Created(e) => {
                let op_seq = self.adapter.get_checkpoint_op_seq(&e.collection_id)?;

                self.state_by_collection_id.insert(
                    e.collection_id.clone(),
                    Arc::new(CollectionState::new(
                        &e.collection_id,
                        self.checkpoint_interval,
                        op_seq,
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
