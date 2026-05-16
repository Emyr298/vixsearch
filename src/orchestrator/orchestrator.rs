use crate::{collection, error::Error, orchestrator::CreateCollectionParam, query, translog};

pub trait Orchestrator {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error>;
    fn execute(&self, query: query::Query) -> Result<(), Error>;
}

struct OrchestratorImpl {
    collection_manager: Box<dyn collection::Manager>,
    translog_manager: Box<dyn translog::Manager>,
}

pub fn new_orchestrator(
    collection_manager: Box<dyn collection::Manager>,
    translog_manager: Box<dyn translog::Manager>,
) -> Box<dyn Orchestrator> {
    return Box::new(OrchestratorImpl::new(collection_manager, translog_manager));
}

impl OrchestratorImpl {
    pub fn new(
        collection_manager: Box<dyn collection::Manager>,
        translog_manager: Box<dyn translog::Manager>,
    ) -> Self {
        return OrchestratorImpl {
            collection_manager,
            translog_manager,
        };
    }

    fn insert(&self, collection: String, data: query::InsertData) -> Result<(), Error> {
        self.collection_manager
            .validate_payload(collection.as_str(), &data.payload)?;

        self.translog_manager.write_insert(collection, data)?;

        Ok(())
    }
}

impl Orchestrator for OrchestratorImpl {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error> {
        let coll_param = collection::CreateCollectionParam::try_from(param)?;
        self.collection_manager.create_collection(coll_param)?;
        Ok(())
    }

    fn execute(&self, query: query::Query) -> Result<(), Error> {
        match query.action {
            query::Action::Insert(insert_data) => self.insert(query.collection, insert_data),
        }
    }
}
