use std::sync::Arc;

use crate::{document::OpSeqCommitEvent, storage::Storage, translog::{GetTransactionsParam, GetTransactionsResult, InsertPortParam, LoadManyResult, LoadOrCreateResult, TranslogPort}, utils::{observer::observer::Observer, vixerr::Error}};

pub const STORE: &str = "translog";
pub const NAME: &str = "append.log";

// TODO: cleanup mechanism
pub struct TranslogAdapter {
    storage: Arc<dyn Storage>,
}

impl TranslogAdapter {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        TranslogAdapter {
            storage,
        }
    }
}

impl TranslogPort for TranslogAdapter {
    fn insert(&self, param: InsertPortParam) -> Result<(), Error> {
        let writer = self.storage.create_append_only_writer(store, name, last_offset)?;

        writer.write()

        todo!()
    }
    
    fn get_transactions(&self, param: GetTransactionsParam) -> Result<GetTransactionsResult, Error> {
        todo!()
    }
    
    fn commit_op_seq(&self, op_seq: u64) -> Result<(), Error> {
        todo!()
    }
    
    fn load_or_create(&self, collection_id: &str) -> Result<LoadOrCreateResult, Error> {
        todo!()
    }
    
    fn load_many(&self, collection_ids: &[String]) -> Result<LoadManyResult, Error> {
        todo!()
    }
    
    fn unload(&self, collection_id: &str) -> Result<(), Error> {
        todo!()
    }
}
