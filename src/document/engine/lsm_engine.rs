use std::{hint::spin_loop, sync::{Arc, Mutex, atomic::Ordering}};

use dashmap::DashMap;
use uuid::Uuid;

use crate::{collection::CollectionLifecycleEvent, document::engine::{COMMIT_IN_PROGRESS, engine::{DocumentEngine, LSMDocumentPort}, entity::OpSeqCommitEngineEvent, errors::{COLLECTION_NOT_FOUND, DOCUMENT_NOT_FOUND}, lsm_entity::{CollectionStateDropEvent, SegmentStateDropEvent}, lsm_state::{CollectionState, SegmentState}}, errcode::{self, FATAL_ERROR}, utils::{handler::handler::Handler, observer::{observer::{Observer, ObserverGroupReader}, observer_group::ObserverGroup}, vixalg, vixerr::Error, vixpool::{POOL_QUEUE_FULL, Pool}}};

pub struct LSMDocumentEngine {
    adapter: Arc<dyn LSMDocumentPort>,
    collection_by_id: DashMap<String, Arc<CollectionState>>,
    segment_by_id: Arc<DashMap<String, Arc<SegmentState>>>,
    flush_pool: Arc<dyn Pool>,
    flush_byte_size_threshold: usize,
    collection_drop_observers: Arc<dyn ObserverGroupReader<CollectionStateDropEvent>>,
    segment_drop_observers: Arc<dyn ObserverGroupReader<SegmentStateDropEvent>>,
    op_seq_commit_engine_observers: Arc<dyn ObserverGroupReader<OpSeqCommitEngineEvent>>,
}

impl LSMDocumentEngine {
    pub fn new(
        adapter: Arc<dyn LSMDocumentPort>,
        flush_pool: Arc<dyn Pool>,
        flush_threshold: usize,
        op_seq_commit_engine_observers: Arc<dyn ObserverGroupReader<OpSeqCommitEngineEvent>>,
    ) -> (Arc<dyn DocumentEngine>, Arc<dyn Handler<CollectionLifecycleEvent>>) {
        let (collection_drop_observers_reader, collection_drop_observers_writer) = ObserverGroup::new(Vec::<Arc<dyn Observer<CollectionStateDropEvent>>>::new());
        let (segment_drop_observers_reader, segment_drop_observers_writer) = ObserverGroup::new(Vec::<Arc<dyn Observer<SegmentStateDropEvent>>>::new());

        let lsm_engine = Arc::new(LSMDocumentEngine {
            adapter,
            collection_by_id: DashMap::new(),
            segment_by_id: Arc::new(DashMap::new()),
            flush_pool,
            flush_byte_size_threshold: flush_threshold,
            collection_drop_observers: collection_drop_observers_reader,
            segment_drop_observers: segment_drop_observers_reader,
            op_seq_commit_engine_observers,
        });

        collection_drop_observers_writer.register(lsm_engine.clone());
        segment_drop_observers_writer.register(lsm_engine.clone());

        let engine: Arc<dyn DocumentEngine> = lsm_engine.clone();
        let lifecycle_handler: Arc<dyn Handler<CollectionLifecycleEvent>> = lsm_engine;
        let collection_drop_observer: Arc<dyn Observer<CollectionStateDropEvent>>;

        return (engine, lifecycle_handler);
    }
}

impl Handler<CollectionLifecycleEvent> for LSMDocumentEngine {
    fn handle(&self, event: &CollectionLifecycleEvent) -> Result<(), Error> {
        match event {
            CollectionLifecycleEvent::Loaded(e) => {
                for collection_id in &e.collection_ids {
                    let result = self.adapter.get_all_segment_by_collection_id(&collection_id)?;

                    let collection_state = Arc::new(CollectionState::from_get_all_segment_by_collection_id_port_result(&collection_id, result, self.collection_drop_observers.clone()));
                    self.collection_by_id.insert(collection_id.to_string(), collection_state.clone());

                    for segment_id in &collection_state.get_segment_ids() {
                        let metadata = self.adapter.get_metadata(&collection_id, segment_id)?;

                        let segment_state = metadata.segment_state(segment_id, &collection_id, self.segment_drop_observers.clone());
                        self.segment_by_id.insert(segment_id.to_string(), Arc::new(segment_state));
                    }
                }

                Ok(())
            },

            CollectionLifecycleEvent::Created(e) => {
                let collection_state = Arc::new(CollectionState::new(&e.collection_id, self.collection_drop_observers.clone()));
                self.collection_by_id.insert(e.collection_id.to_string(), collection_state);

                Ok(())
            },

            // this method stops new CRUD queries on collection/segment, but old one that already
            // kept their references (arc) still continues and will be closed on all arc drops
            CollectionLifecycleEvent::Deleted(e) => {
                self.collection_by_id.remove(&e.collection_id);
                
                Ok(())
            },
        }
    }
}

impl Observer<CollectionStateDropEvent> for LSMDocumentEngine {
    fn observe(&self, event: &CollectionStateDropEvent) {
        for segment_id in &event.segment_ids {
            self.segment_by_id.remove(segment_id);
        }
    }
}

impl Observer<SegmentStateDropEvent> for LSMDocumentEngine {
    fn observe(&self, event: &SegmentStateDropEvent) {
        if let Err(e) = self.adapter.archive_segment(&event.collection_id, &event.id) {
            println!("failed to archive segment {}: {}", &event.id, e.message)
        }
    }
}

impl DocumentEngine for LSMDocumentEngine {
    fn get_by_key(&self, collection_id: &str, key: &[u8]) -> Result<Vec<u8>, Error> {
        let Some(collection) = self.collection_by_id.get(collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id))
                .throw();
        };

        if let Some(value) = collection.value_from_buffer(key) {
            return Ok(value);
        }

        for segment_id in collection.get_segment_ids() {
            let Some(segment) = self.segment_by_id.get(&segment_id).map(|s| Arc::clone(s.value())) else {
                return Error::code(errcode::FATAL_ERROR)
                    .message(format!("segment {} not found", &segment_id))
                    .throw();
            };

            if !segment.may_contain_key(key) {
                continue;
            }

            match self.get_value_from_segment(&segment, key) {
                Ok(doc) => return Ok(doc),
                Err(err) => {
                    if err.code == DOCUMENT_NOT_FOUND {
                        continue;
                    }
                    return Err(err);
                }
            };
        }

        Error::code(DOCUMENT_NOT_FOUND)
            .message(format!("document {} not found", String::from_utf8_lossy(key)))
            .throw()
    }

    fn insert(&self, collection_id: &str, key: &[u8], value: &[u8], op_seq: u64) -> Result<(), Error> {
        let Some(collection) = self.collection_by_id.get(collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id))
                .throw();
        };

        collection.insert_buffer_value(key, value, op_seq);

        if let Err(err) = self.should_flush(collection) {
            // let the next inserts trigger the flush when queue full
            if err.code != POOL_QUEUE_FULL {
                return err.throw();
            }
        }
        
        Ok(())
    }
}

impl LSMDocumentEngine {
    fn get_value_from_segment(&self, segment: &SegmentState, key: &[u8]) -> Result<Vec<u8>, Error> {
        let Some(value) = vixalg::binary_search(&segment.key_block_offsets, |block_offset| {
            self.get_value_from_segment_compare_fn(segment, block_offset, key)
        })? else {
            return Error::code(DOCUMENT_NOT_FOUND)
                .message(format!(
                    "value {} not found in segment {}",
                    String::from_utf8_lossy(key),
                    segment.id
                ))
                .throw();
        };

        Ok(value)
    }

    fn get_value_from_segment_compare_fn(&self, segment: &SegmentState, block_offset: &u64, key: &[u8]) -> Result<vixalg::Ordering<Vec<u8>>, Error> {
        let values = self.adapter.get_values_from_block(&segment.collection_id, &segment.id, *block_offset)?;
        if values.len() == 0 {
            return Error::code(errcode::FATAL_ERROR)
                .message(format!("segment {} has no value in block {}", &segment.id, block_offset))
                .throw();
        }

        let first_key = values[0].0.as_slice();
        if key < first_key {
            return Ok(vixalg::Ordering::Less);
        }

        let last_key = values[values.len() - 1].0.as_slice();
        if key > last_key {
            return Ok(vixalg::Ordering::Greater);
        }

        for (value_key, value) in values {
            if value_key.as_slice() == key {
                return Ok(vixalg::Ordering::Equal(value));
            }
        }

        Ok(vixalg::Ordering::NotFound)
    }

    fn should_flush(&self, collection: Arc<CollectionState>) -> Result<(), Error> {
        let adapter = self.adapter.clone();
        let segment_by_id = self.segment_by_id.clone();
        let op_seq_flush_observers = self.op_seq_commit_engine_observers.clone();
        let flush_byte_size_threshold = self.flush_byte_size_threshold;
        let segment_drop_observers = self.segment_drop_observers.clone();
        
        self.flush_pool.submit(Box::new(move || {
            let commit_session = match collection.try_begin_commit(flush_byte_size_threshold) {
                Some(cs) => cs,
                None => return,
            };

            let segment_id = Uuid::new_v4().to_string();
            let segment_metadata = match adapter.flush_segment(&collection.id, &segment_id, commit_session.sorted_committed_key_values()) {
                Ok(metadata) => metadata,
                Err(e) => {
                    eprintln!("failed to flush collection {}: {}", collection.id, e);
                    // TODO: handle FATAL_ERROR after handling is defined
                    return;
                },
            };

            segment_by_id.insert(segment_id.clone(), Arc::new(segment_metadata.segment_state(&segment_id, &collection.id, segment_drop_observers)));
            collection.insert_segment(&segment_id, 0);

            let event = commit_session.op_seq_commit_engine_event();
            commit_session.finalize();
            
            for observer in op_seq_flush_observers.list().iter() {
                observer.observe(&event);
            }
        }))?;

        Ok(())
    }
}
