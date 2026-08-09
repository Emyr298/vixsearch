use std::sync::{Arc, Mutex, atomic::Ordering};

use dashmap::DashMap;

use crate::{document::engine::{engine::{Engine, LSMPort}, errors::{COLLECTION_NOT_FOUND, DOCUMENT_NOT_FOUND}, lsm_state::{CollectionState, SegmentState}}, errcode, utils::{vixalg, vixerr::Error, vixpool::{POOL_QUEUE_FULL, Pool}}};

pub struct LSMEngine {
    adapter: Arc<dyn LSMPort>,
    collections_by_id: DashMap<String, Arc<CollectionState>>,
    segments_by_id: DashMap<String, Arc<SegmentState>>,
    flush_pool: Arc<dyn Pool>,
    flush_threshold: usize,
    flush_lock: Arc<Mutex<()>>,
}

pub fn new_lsm_engine(adapter: Arc<dyn LSMPort>, flush_pool: Arc<dyn Pool>, flush_threshold: usize) -> Arc<dyn Engine> {
    return Arc::new(LSMEngine::new(adapter, flush_pool, flush_threshold));
}

impl LSMEngine {
    pub fn new(adapter: Arc<dyn LSMPort>, flush_pool: Arc<dyn Pool>, flush_threshold: usize) -> Self {
        LSMEngine {
            adapter,
            collections_by_id: DashMap::new(),
            segments_by_id: DashMap::new(),
            flush_pool,
            flush_threshold,
            flush_lock: Arc::new(Mutex::new(())),
        }
    }
}

impl Engine for LSMEngine {
    fn get_by_key(&self, collection_id: &str, key: &[u8]) -> Result<Vec<u8>, crate::utils::vixerr::Error> {
        let Some(collection) = self.collections_by_id.get(collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id))
                .throw();
        };

        if let Some(value) = collection.value_from_buffer(key) {
            return Ok(value);
        }

        for segment_id in collection.get_segment_ids() {
            let Some(segment) = self.segments_by_id.get(&segment_id).map(|s| Arc::clone(s.value())) else {
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
            .message(format!(
                "document {} not found",
                String::from_utf8_lossy(key),
            ))
            .throw()
    }

    fn insert(&self, collection_id: &str, key: &[u8], value: &[u8]) -> Result<(), Error> {
        let Some(collection) = self.collections_by_id.get(collection_id).map(|c| Arc::clone(c.value())) else {
            return Error::code(COLLECTION_NOT_FOUND)
                .message(format!("collection {} not found", collection_id))
                .throw();
        };

        let buffer = loop {
            let candidate = collection.buffer.load();
            candidate.add_in_flight();

            // ensures after committed, we can't add in_flight anymore
            if Arc::ptr_eq(&candidate, &collection.buffer.load()) {
                break candidate;
            }

            candidate.sub_in_flight();
        };

        buffer.insert(key.to_vec(), value.to_vec());
        buffer.sub_in_flight();

        if let Err(err) = self.should_flush(collection) {
            // let the next inserts trigger the flush
            if err.code != POOL_QUEUE_FULL {
                return err.throw();
            }
        }
        
        Ok(())
    }
}

impl LSMEngine {
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
        let values = self.adapter.get_values_from_block(&segment.id, *block_offset)?;
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
        let buffer = collection.buffer.load();
        let byte_size = buffer.byte_size.load(Ordering::Relaxed);
        
        if byte_size < self.flush_threshold {
            return Ok(());
        }

        let adapter = Arc::clone(&self.adapter);
        let flush_lock = Arc::clone(&self.flush_lock);
        self.flush_pool.submit(Box::new(move || {
            let _guard = match flush_lock.try_lock() {
                Ok(val) => val,
                Err(_) => return,
            };

            if let Err(err) = collection.start_flush() {
                eprintln!("failed to flush collection {}: {}", collection.id, err);
                // TODO: handle FATAL_ERROR after handling is defined
                return;
            }

            let commit_buffer_opt = collection.commit_buffer.load();
            if let Some(commit_buffer) = commit_buffer_opt.as_ref() {
                if let Err(err) = adapter.flush_segment(Arc::clone(commit_buffer)) {
                    eprintln!("failed to flush collection {}: {}", collection.id, err);
                    // TODO: handle FATAL_ERROR after handling is defined
                }
            }

            collection.end_flush();
        }))?;

        Ok(())
    }
}