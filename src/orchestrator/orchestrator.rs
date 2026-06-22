use std::{iter::once, sync::{Arc, RwLock}};

use dashmap::DashMap;

use crate::{collection, errcode, index, orchestrator::{CreateCollectionParam, CreateCollectionParamField}, shared::{Document, get_id}, vixerr::Error};

pub trait Orchestrator: Send + Sync {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error>;
    fn delete_collection(&self, name: &str) -> Result<(), Error>;
    fn load_collection(&self) -> Result<(), Error>;

    fn get_document(&self, collection: &str, id: &str) -> Result<Document, Error>;
    fn insert_document(&self, collection: String, document: Document) -> Result<(), Error>;
}

struct OrchestratorImpl {
    collection_manager: Box<dyn collection::Manager>,
    index_manager: Box<dyn index::Manager>,

    id_locks: DashMap<String, Arc<RwLock<()>>>,
}

pub fn new_orchestrator(
    collection_manager: Box<dyn collection::Manager>,
    index_manager: Box<dyn index::Manager>,
) -> Box<dyn Orchestrator> {
    return Box::new(OrchestratorImpl::new(collection_manager, index_manager));
}

impl OrchestratorImpl {
    pub fn new(
        collection_manager: Box<dyn collection::Manager>,
        index_manager: Box<dyn index::Manager>,
    ) -> Self {
        return OrchestratorImpl {
            collection_manager,
            index_manager,
            id_locks: DashMap::new(),
        };
    }
}

impl OrchestratorImpl {
    fn get_or_create_id_lock(&self, id: &str) -> Arc<RwLock<()>> {
        self.id_locks
            .entry(id.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(())))
            .clone()
    }
}

impl Orchestrator for OrchestratorImpl {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error> {
        param.validate()?;

        let enriched_param = CreateCollectionParam {
            fields: once(CreateCollectionParamField::identifier())
                .chain(param.fields)
                .collect(),
            ..param
        };

        for field in &enriched_param.fields {
            self.index_manager.validate_field(&field.field_type, field.index_type)?;
        }

        let coll_param = collection::CreateCollectionParam::try_from(&enriched_param)?;
        self.collection_manager.create_collection(coll_param)?;

        self.index_manager.create(enriched_param.index_create_param())
    }

    fn delete_collection(&self, name: &str) -> Result<(), Error> {
        self.collection_manager.delete_collection(name)?;
        Ok(())
    }
    
    fn load_collection(&self) -> Result<(), Error> {
        println!("Loading data...");
        self.collection_manager.load().expect("failed to load collection");
        println!("Data loaded");
        Ok(())
    }

    fn get_document(&self, collection: &str, id: &str) -> Result<Document, Error> {
        let lock = self.get_or_create_id_lock(id);
        let _guard = lock.read().unwrap();

        self.index_manager.get(index::GetParam { collection, id })
    }

    fn insert_document(
        &self,
        collection: String,
        document: Document,
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

        let id = get_id(&document)?;
        let lock = self.get_or_create_id_lock(&id);
        let _guard = lock.write().unwrap();

        self.index_manager.insert(index::InsertParam { collection: &collection, document: &document })
    }
}
