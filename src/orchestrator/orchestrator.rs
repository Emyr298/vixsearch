use std::{collections::HashMap, iter::once};

use crate::{collection, errcode, index, orchestrator::{CreateCollectionParam, CreateCollectionParamField}, shared, vixerr::Error};

pub trait Orchestrator: Send + Sync {
    fn create_collection(&self, param: CreateCollectionParam) -> Result<(), Error>;
    fn delete_collection(&self, name: &str) -> Result<(), Error>;
    fn load_collection(&self) -> Result<(), Error>;

    fn insert_document(
        &self,
        collection: String,
        document: HashMap<String, shared::Value>,
    ) -> Result<(), Error>;
}

struct OrchestratorImpl {
    collection_manager: Box<dyn collection::Manager>,
    index_manager: Box<dyn index::Manager>,
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
        };
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

        // TODO: what if index is partially created/updated? whole collection must be locked or queue (async indexing) or smthZ
        for field in &enriched_param.fields {
            self.index_manager.create(field.index_create_param(&enriched_param.name))?;
        }

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

        // TODO: what if some indexes fail?
        // TODO: parallelize
        for (field, value) in document {
            self.index_manager.insert(param);
        }

        Ok(())
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
}
