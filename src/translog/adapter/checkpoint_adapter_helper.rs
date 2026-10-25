use serde::{Deserialize, Serialize};

pub fn checkpoint_store(collection_id: &str) -> String {
    format!("collection/{}/checkpoint", collection_id)
}

#[derive(Serialize, Deserialize)]
pub struct Checkpoint {
    pub op_seq: u64,
}

impl Checkpoint {
    pub fn new(op_seq: u64) -> Self {
        Checkpoint { op_seq }
    }
}
