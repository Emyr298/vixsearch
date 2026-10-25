use crate::{
    storage::Storage,
    translog::{
        CheckpointPort,
        adapter::{
            checkpoint_adapter_helper::{Checkpoint, checkpoint_store},
            commit_helper::{CommitStorage, CommitStorageImpl},
        },
    },
    utils::vixerr::Error,
};
use std::{cmp::max, sync::Arc};

pub struct CheckpointAdapter {
    commit_storage: Arc<dyn CommitStorage<Checkpoint>>,
}

impl CheckpointAdapter {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        CheckpointAdapter {
            commit_storage: Arc::new(CommitStorageImpl::new(storage)),
        }
    }
}

impl CheckpointPort for CheckpointAdapter {
    fn get_checkpoint_op_seq(&self, collection_id: &str) -> Result<u64, Error> {
        let store = checkpoint_store(collection_id);
        let checkpoint = match self.commit_storage.get_last_commit(&store)?.commit {
            Some(c) => c,
            None => return Ok(0),
        };
        Ok(checkpoint.op_seq)
    }

    fn submit_checkpoint(&self, collection_id: &str, op_seq: u64) -> Result<(), Error> {
        let store = checkpoint_store(collection_id);
        let result = self.commit_storage.get_last_commit(&store)?;

        if let Some(c) = &result.commit
            && c.op_seq >= op_seq
        {
            return Ok(());
        }

        self.commit_storage
            .submit(&store, result.next_number, Checkpoint::new(op_seq))
    }
}
