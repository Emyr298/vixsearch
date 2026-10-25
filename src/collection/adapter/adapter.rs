use std::sync::{Arc, RwLock};

use log::info;

use crate::collection::adapter::helper::{
    COMMIT_STORE, SchemaBlock, SchemaCommit, schema_store_name,
};
use crate::collection::port_param_result::{
    CreatePortParam, GetAllPortResult, GetAllPortResultCollection,
};
use crate::collection::service::CollectionPort;
use crate::storage::Storage;
use crate::utils::commit::commit_storage::CommitStorage;
use crate::utils::commit::commit_storage_impl::CommitStorageImpl;
use crate::utils::vixerr::Error;

pub struct CollectionAdapter {
    storage: Arc<dyn Storage>,
    commit_storage: Arc<dyn CommitStorage<SchemaCommit>>,
    commit_lock: RwLock<()>,
}

impl CollectionAdapter {
    pub fn new(storage: Arc<dyn Storage>) -> Arc<dyn CollectionPort> {
        Arc::new(CollectionAdapter {
            storage: storage.clone(),
            commit_storage: CommitStorageImpl::new(storage),
            commit_lock: RwLock::new(()),
        })
    }
}

impl CollectionPort for CollectionAdapter {
    fn get_all(&self) -> Result<GetAllPortResult, Error> {
        let _guard = self.commit_lock.read().unwrap();

        let result = self.commit_storage.get_last_commit(COMMIT_STORE)?;
        let internal_ids = result
            .commit
            .map(|c| c.collection_internal_ids)
            .unwrap_or(Vec::new());

        let mut collections: Vec<GetAllPortResultCollection> = Vec::new();
        for internal_id in internal_ids {
            let (schema_store, schema_name) = schema_store_name(&internal_id);
            let reader = self
                .storage
                .get_or_create_reader(&schema_store, &schema_name)?;

            let schema_buf = reader.read_all()?;
            let schema_block = SchemaBlock::from_buf(schema_buf)?;
            let schema = match schema_block.decode() {
                Ok(s) => s,
                Err(e) => return e.throw(),
            };

            collections.push(schema.get_all_port_result_collection()?);
        }

        Ok(GetAllPortResult { collections })
    }

    fn create(&self, param: CreatePortParam) -> Result<(), Error> {
        let internal_id = param.internal_id.clone();
        let (store, name) = schema_store_name(&internal_id);
        let schema_buf = SchemaBlock::from_create_port_param(param).encode();

        let mut schema_writer = self.storage.create_write_once_writer(&store, &name)?;
        schema_writer.write(&schema_buf)?;
        schema_writer.commit()?;

        let _guard = self.commit_lock.write().unwrap();

        let result = self.commit_storage.get_last_commit(COMMIT_STORE)?;
        let schema_commit_data = SchemaCommit::add_collection(&internal_id, &result);

        self.commit_storage
            .submit(COMMIT_STORE, result.next_number, schema_commit_data)
    }

    fn delete_by_internal_id(&self, internal_id: &str) -> Result<(), Error> {
        let _guard = self.commit_lock.write().unwrap();

        let result = self.commit_storage.get_last_commit(COMMIT_STORE)?;
        if let Some(schema_commit_data) = SchemaCommit::delete_collection(internal_id, &result) {
            self.commit_storage
                .submit(COMMIT_STORE, result.next_number, schema_commit_data)?;
        }

        Ok(())
    }
}
