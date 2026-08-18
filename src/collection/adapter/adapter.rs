use std::sync::{Arc, RwLock};

use crate::collection::adapter::helper::{COMMIT_STORE, Commit, CommitBlock, GetLatestCommitResult, SchemaBlock, next_commit_number, schema_store_name};
use crate::collection::port_param_result::{CreatePortParam, GetAllPortResult, GetAllPortResultCollection};
use crate::collection::service::Port;
use crate::errcode::PARSE_ERROR;
use crate::storage::WormStorage;
use crate::utils::vixerr::Error;

struct Adapter {
    storage: Arc<dyn WormStorage>,
    commit_lock: RwLock<()>,
}

impl Adapter {
    fn new(storage: Arc<dyn WormStorage>) -> Arc<dyn Port> {
        Arc::new(Adapter {
            storage: storage,
            commit_lock: RwLock::new(()),
        })
    }

    // TODO: invalid commits should be cleaned at load
    fn get_latest_commit(&self) -> Result<GetLatestCommitResult, Error> {
        let names = self.storage.get_all_commited_name_sorted(COMMIT_STORE)?;
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

impl Port for Adapter {
    fn get_all(&self) -> Result<GetAllPortResult, Error> {
        let _guard = self.commit_lock.read().unwrap();

        let result = self.get_latest_commit()?;
        let collection_ids = result.latest_commit.map(|c| c.collection_ids).unwrap_or(vec![]);

        let mut collections: Vec<GetAllPortResultCollection> = Vec::new();
        for collection_id in collection_ids {
            let (schema_store, schema_name) = schema_store_name(&collection_id);
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
        let collection_id = param.id.clone();
        let (store, name) = schema_store_name(&param.id);
        let schema_buf = SchemaBlock::from_create_port_param(param).encode();

        let mut schema_writer = self.storage.get_writer(&store, &name)?;
        schema_writer.write(&schema_buf)?;
        schema_writer.commit()?;

        let _guard = self.commit_lock.write().unwrap();

        let result = self.get_latest_commit()?;
        let commit = Commit::add_collection(&collection_id, result);

        let mut commit_writer = self.storage.get_writer(COMMIT_STORE, &commit.name())?;
        let commit_buf = CommitBlock::from_commit(commit).encode();

        commit_writer.write(&commit_buf)?;
        commit_writer.commit()
    }

    fn delete(&self, id: &str) -> Result<(), Error> {
        let _guard = self.commit_lock.write().unwrap();

        let result = self.get_latest_commit()?;
        if let Some(commit) = Commit::delete_collection(id, result) {
            let mut commit_writer = self.storage.get_writer(COMMIT_STORE, &commit.name())?;
            let commit_buf = CommitBlock::from_commit(commit).encode();

            commit_writer.write(&commit_buf)?;
            commit_writer.commit()?;
        }

        Ok(())
    }
}
