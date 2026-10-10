use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use crate::{document::engine::OpSeqCommitEngineEvent, errcode::PARSE_ERROR, utils::vixerr::Error};

pub fn key_from_id(id: &str) -> Vec<u8> {
    format!("id_{}", id).into_bytes()
}

pub fn key_from_seq_id(seq_id: &u64) -> Vec<u8> {
    [b"seq_", &seq_id.to_le_bytes()[..]].concat()
}


#[derive(Clone)]
pub struct OpSeqCommitEvent {
    pub collection_id: String,
    pub op_seq: u64,
}

impl OpSeqCommitEvent {
    pub fn new(engine_event: &OpSeqCommitEngineEvent) -> Self {
        OpSeqCommitEvent {
            collection_id: engine_event.collection_id.clone(),
            op_seq: engine_event.op_seq,
        }
    }
}
