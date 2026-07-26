use std::{sync::{Arc, mpsc::{self, Receiver, Sender}}, thread};

use dashmap::DashMap;

use crate::{document::engine::{engine::{Engine, Port}, errors::{COLLECTION_NOT_FOUND, DOCUMENT_NOT_FOUND}, lsm_entity::Operation, lsm_state::{CollectionState, SegmentState}, param_result::InsertParam}, errcode::{self, FATAL_ERROR}, utils::{vixalg, vixerr::Error}};

pub struct LSMEngine {
    adapter: Arc<dyn Port>,
    collections_by_id: DashMap<String, Arc<CollectionState>>,
    segments_by_id: DashMap<String, Arc<SegmentState>>,
    op_chan_sender: Sender<Vec<Operation>>,
    op_batch_number: i32,
}

pub fn new_lsm_engine(adapter: Arc<dyn Port>, op_batch_number: i32) -> (Arc<dyn Engine>, impl FnOnce() + Send + 'static) {
    let (op_chan_sender, op_chan_receiver) = mpsc::channel::<Vec<Operation>>();

    let engine: Arc<LSMEngine> = Arc::new(LSMEngine::new(adapter, op_chan_sender, op_batch_number));
    let listen_operations_fn = listen_lsm_operations(Arc::clone(&engine), op_chan_receiver);

    return (engine, listen_operations_fn);
}

fn listen_lsm_operations(engine: Arc<LSMEngine>, op_chan_receiver: Receiver<Vec<Operation>>) -> impl FnOnce() + Send + 'static {
    || {
        thread::spawn(move || {
            loop {
                let first_op = match op_chan_receiver.recv() {
                    Ok(op) => op,
                    Err(_) => break,
                };

                let mut batch = vec![first_op];
                while let Ok(op) = op_chan_receiver.try_recv() {
                    batch.push(op);
                }

                engine.execute_operations(batch.into_iter().flatten().collect())?;
            }
        });
    }
}

impl LSMEngine {
    pub fn new(adapter: Arc<dyn Port>, op_chan_sender: Sender<Vec<Operation>>, op_batch_number: i32) -> Self {
        LSMEngine {
            adapter,
            collections_by_id: DashMap::new(),
            segments_by_id: DashMap::new(),
            op_chan_sender,
            op_batch_number,
        }
    }
}

impl Engine for LSMEngine {
    fn get_by_key(&self, collection_id: &str, key: &str) -> Result<Vec<u8>, crate::utils::vixerr::Error> {
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
            .message(format!("document {} not found", key))
            .throw()
    }

    fn insert(&self, collection_id: &str, param: InsertParam) -> Result<(), Error> {
        if let Err(err) = self.op_chan_sender.send(vec![Operation {
            collection_id: collection_id.to_string(),
            key: param.key,
            op_seq: param.op_seq,
            value: param.value,
        }]) {
            return Error::code(FATAL_ERROR)
                .message("failed to send operation into channel")
                .wrap(err)
                .throw();
        }

        // TODO: wait until commit number

        Ok(())
    }

    fn batch_insert(&self, collection_id: &str, params: Vec<InsertParam>) -> Result<(), Error> {
        let operations: Vec<Operation> = params
            .into_iter()
            .map(|param| Operation {
                collection_id: collection_id.to_string(),
                key: param.key,
                op_seq: param.op_seq,
                value: param.value,
            })
            .collect();

        if let Err(err) = self.op_chan_sender.send(operations) {
            return Error::code(FATAL_ERROR)
                .message("failed to send operation into channel")
                .wrap(err)
                .throw();
        }

        // TODO: wait until commit number

        Ok(())
    }

    fn flush(&self, _: &str) -> Result<(), crate::utils::vixerr::Error> {
        todo!()
    }
}

impl LSMEngine {
    fn execute_operations(&self, operations: Vec<Operation>) -> Result<(), Error> {
        Ok(())
    }

    fn get_value_from_segment(&self, segment: &SegmentState, key: &str) -> Result<Vec<u8>, Error> {
        let Some(value) = vixalg::binary_search(&segment.key_block_offsets, |block_offset| {
            self.get_value_from_segment_compare_fn(segment, block_offset, key)
        })? else {
            return Error::code(DOCUMENT_NOT_FOUND)
                .message(format!("value {} not found in segment {}", key, segment.id))
                .throw();
        };

        Ok(value)
    }

    fn get_value_from_segment_compare_fn(&self, segment: &SegmentState, block_offset: &u64, key: &str) -> Result<vixalg::Ordering<Vec<u8>>, Error> {
        let values = self.adapter.get_values_from_block(&segment.id, *block_offset)?;
        if values.len() == 0 {
            return Error::code(errcode::FATAL_ERROR)
                .message(format!("segment {} has no value in block {}", &segment.id, block_offset))
                .throw();
        }

        let first_key = values[0].0.as_str();
        if key < first_key {
            return Ok(vixalg::Ordering::Less);
        }

        let last_key = values[values.len() - 1].0.as_str();
        if key > last_key {
            return Ok(vixalg::Ordering::Greater);
        }

        for (value_key, value) in values {
            if value_key.as_str() == key {
                return Ok(vixalg::Ordering::Equal(value));
            }
        }

        Ok(vixalg::Ordering::NotFound)
    }
}