use std::collections::HashMap;

use crate::{collection, errcode, orchestrator::CreateCollectionParam, shared, vixerr::Error};

pub trait Orchestrator: Send + Sync {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error>;
    fn delete_collection(&self, name: &str) -> Result<(), Error>;

    fn insert_document(
        &self,
        collection: String,
        document: HashMap<String, shared::Value>,
    ) -> Result<(), Error>;
}

struct OrchestratorImpl {
    collection_manager: Box<dyn collection::Manager>,
}

pub fn new_orchestrator(collection_manager: Box<dyn collection::Manager>) -> Box<dyn Orchestrator> {
    return Box::new(OrchestratorImpl::new(collection_manager));
}

impl OrchestratorImpl {
    pub fn new(collection_manager: Box<dyn collection::Manager>) -> Self {
        return OrchestratorImpl { collection_manager };
    }
}

impl Orchestrator for OrchestratorImpl {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error> {
        let coll_param = collection::CreateCollectionParam::try_from(param)?;
        println!("{:?}", coll_param);
        self.collection_manager.create_collection(coll_param)?;
        Ok(())
    }

    fn insert_document(
        &self,
        collection: String,
        document: HashMap<String, shared::Value>,
    ) -> Result<(), Error> {
        let is_valid = self
            .collection_manager
            .validate_payload(collection.as_str(), &document)?;

        if !is_valid {
            return Err(Error::new(
                errcode::PARSE_ERROR,
                "Document validation failed",
            ));
        }

        Ok(())
    }

    fn delete_collection(&self, name: &str) -> Result<(), Error> {
        self.collection_manager.delete_collection(name)?;
        Ok(())
    }
}
