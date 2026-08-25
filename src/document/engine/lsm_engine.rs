use std::{hint::spin_loop, sync::{Arc, Mutex, atomic::Ordering}};

use dashmap::DashMap;
use uuid::Uuid;

use crate::{document::engine::{COMMIT_IN_PROGRESS, DocumentEngineLoader, engine::{DocumentEngine, LSMDocumentPort}, errors::{COLLECTION_NOT_FOUND, DOCUMENT_NOT_FOUND}, lsm_state::{CollectionState, SegmentState}}, errcode, utils::{vixalg, vixerr::Error, vixpool::{POOL_QUEUE_FULL, Pool}}};

pub struct LSMDocumentEngine {
    adapter: Arc<dyn LSMDocumentPort>,
    collection_by_id: DashMap<String, Arc<CollectionState>>,
    segment_by_id: Arc<DashMap<String, Arc<SegmentState>>>,
    flush_pool: Arc<dyn Pool>,
    flush_byte_size_threshold: usize,
}

impl LSMDocumentEngine {
    pub fn new(
        adapter: Arc<dyn LSMDocumentPort>,
        flush_pool: Arc<dyn Pool>,
        flush_threshold: usize,
    ) -> (Arc<dyn DocumentEngine>, Arc<dyn DocumentEngineLoader>) {
        let lsm_engine = Arc::new(LSMDocumentEngine {
            adapter,
            collection_by_id: DashMap::new(),
            segment_by_id: Arc::new(DashMap::new()),
            flush_pool,
            flush_byte_size_threshold: flush_threshold,
        });

        let engine: Arc<dyn DocumentEngine> = lsm_engine.clone();
        let loader: Arc<dyn DocumentEngineLoader> = lsm_engine;

        return (engine, loader);
    }
}

impl DocumentEngineLoader for LSMDocumentEngine {
    fn load(&self, collection_ids: &[String]) -> Result<(), Error> {
        for collection_id in collection_ids {
            let result = self.adapter.get_all_segment_by_collection_id(collection_id)?;

            let collection_state = Arc::new(CollectionState::from_get_all_segment_by_collection_id_port_result(collection_id, result));
            self.collection_by_id.insert(collection_id.to_string(), collection_state.clone());

            for segment_id in &collection_state.get_segment_ids() {
                let metadata = self.adapter.get_metadata(collection_id, segment_id)?;

                let segment_state = metadata.segment_state(segment_id, collection_id);
                self.segment_by_id.insert(segment_id.to_string(), Arc::new(segment_state));
            }
        }

        Ok(())
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

    fn insert(&self, collection_id: &str, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let Some(collection) = self.collection_by_id.get(collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id))
                .throw();
        };

        collection.insert_buffer_value(key, value);

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
        if collection.is_flushable(self.flush_byte_size_threshold) {
            return Ok(());
        }

        let adapter = self.adapter.clone();
        let segment_by_id = self.segment_by_id.clone();
        
        self.flush_pool.submit(Box::new(move || {
            let commit_guard = match collection.try_begin_commit() {
                Some(cg) => cg,
                None => return,
            };

            if commit_guard.buffer.is_empty() {
                return;
            }

            let segment_id = Uuid::new_v4().to_string();
            let segment_metadata = match adapter.flush_segment(&collection.id, &segment_id, commit_guard.buffer.sorted_key_values()) {
                Ok(metadata) => metadata,
                Err(e) => {
                    eprintln!("failed to flush collection {}: {}", collection.id, e);
                    // TODO: handle FATAL_ERROR after handling is defined
                    return;
                },
            };

            segment_by_id.insert(segment_id.clone(), Arc::new(segment_metadata.segment_state(&segment_id, &collection.id)));
            collection.insert_segment(&segment_id, 0);
        }))?;

        Ok(())
    }
}
