use std::sync::{Arc, RwLock};

use crate::collection::adapter::helper::{COMMIT_STORE, Commit, CommitBlock, GetLatestCommitResult, SchemaBlock, next_commit_number, schema_store_name};
use crate::collection::port_param_result::{CreatePortParam, GetAllPortResult, GetAllPortResultCollection};
use crate::collection::service::CollectionPort;
use crate::errcode::PARSE_ERROR;
use crate::storage::Storage;
use crate::utils::vixerr::Error;

pub struct CollectionAdapter {
    storage: Arc<dyn Storage>,
    commit_lock: RwLock<()>,
}

impl CollectionAdapter {
    pub fn new(storage: Arc<dyn Storage>) -> Arc<dyn CollectionPort> {
        Arc::new(CollectionAdapter {
            storage: storage,
            commit_lock: RwLock::new(()),
        })
    }

    // TODO: invalid commits should be cleaned at load
    fn get_latest_commit(&self) -> Result<GetLatestCommitResult, Error> {
        let names = self.storage.get_all_name_sorted(COMMIT_STORE)?;
        let next_commit_number = next_commit_number(&names);

        for commit_name in names.into_iter().filter(|n| n.starts_with("commit_")).rev() {
            match self.get_commit(&commit_name) {
                Ok(c) => return Ok(GetLatestCommitResult::new(next_commit_number, Some(c))),
                Err(e) if e.code != PARSE_ERROR => return Err(e),
                Err(_) => continue,
            }
        }

        Ok(GetLatestCommitResult::new(next_commit_number, None))
    }

    fn get_commit(&self, commit_name: &str) -> Result<Commit, Error> {
        let reader = self.storage.get_reader(COMMIT_STORE, commit_name)?;
        let commit_buf = reader.read_all()?;

        let commit_block = CommitBlock::from_buf(commit_buf)?;
        commit_block.decode()
    }
}

impl CollectionPort for CollectionAdapter {
    fn get_all(&self) -> Result<GetAllPortResult, Error> {
        let _guard = self.commit_lock.read().unwrap();

        let result = self.get_latest_commit()?;
        let internal_ids = result.latest_commit.map(|c| c.collection_internal_ids).unwrap_or(vec![]);

        let mut collections: Vec<GetAllPortResultCollection> = Vec::new();
        for internal_id in internal_ids {
            let (schema_store, schema_name) = schema_store_name(&internal_id);
            let reader = self.storage.get_reader(&schema_store, &schema_name)?;

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

        let mut schema_writer = self.storage.get_write_once_writer(&store, &name)?;
        schema_writer.write(&schema_buf)?;
        schema_writer.commit()?;

        let _guard = self.commit_lock.write().unwrap();

        let result = self.get_latest_commit()?;
        let commit = Commit::add_collection(&internal_id, result);

        let mut commit_writer = self.storage.get_write_once_writer(COMMIT_STORE, &commit.name())?;
        let commit_buf = CommitBlock::from_commit(commit).encode();

        commit_writer.write(&commit_buf)?;
        commit_writer.commit()
    }

    fn delete_by_internal_id(&self, internal_id: &str) -> Result<(), Error> {
        let _guard = self.commit_lock.write().unwrap();

        let result = self.get_latest_commit()?;
        if let Some(commit) = Commit::delete_collection(internal_id, result) {
            let mut commit_writer = self.storage.get_write_once_writer(COMMIT_STORE, &commit.name())?;
            let commit_buf = CommitBlock::from_commit(commit).encode();

            commit_writer.write(&commit_buf)?;
            commit_writer.commit()?;
        }

        Ok(())
    }
}
