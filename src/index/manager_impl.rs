use std::{collections::{HashMap, HashSet}, sync::Arc};

use dashmap::{DashMap};

use crate::{errcode, index::{GetParam, IndexType, InsertParam, SearchParam, field_key, identifier, manager::{Manager, TypeManager}, param_result::{CreateParam, DeleteParam}}, shared::{Document, IDENTIFIER_FIELD, Value, ValueType, get_id}, utils::vixerr::Error};

struct ManagerImpl {
    allowed_index_types: Arc<HashMap<ValueType, HashSet<IndexType>>>,
    identifier_manager: Arc<dyn identifier::Manager>,
    type_managers: DashMap<IndexType, Arc<dyn TypeManager>>,
    index_map: DashMap<String, IndexType>,
}

impl ManagerImpl {
    fn new(
        allowed_index_types: HashMap<ValueType, HashSet<IndexType>>,
        identifier_manager: Arc<dyn identifier::Manager>, 
        type_managers: Vec<(IndexType, Arc<dyn TypeManager>)>
    ) -> Self {
        ManagerImpl {
            allowed_index_types: Arc::new(allowed_index_types),
            identifier_manager: identifier_manager,
            type_managers: type_managers.into_iter().collect(),
            index_map: DashMap::new(),
        }
    }
}

impl Manager for ManagerImpl {
    // TODO: fail from this must be panic from orchestrator
    // TODO: what if index is partially created/updated? whole collection must be locked or queue (async indexing) or smthZ
    fn create(&self, param: CreateParam) -> Result<(), Error> {
        self.identifier_manager.create(identifier::CreateParam {
            collection: param.collection
        })
    }
    
    // TODO: fail from this must be panic from orchestrator
    fn delete(&self, param: DeleteParam) -> Result<(), Error> {
        self.identifier_manager.delete(identifier::DeleteParam {
            collection: param.collection
        })
    }

    fn get(&self, param: GetParam) -> Result<Document, Error> {
        self.identifier_manager.get(identifier::GetParam {
            collection: param.collection,
            id: param.id
        })
    }

    fn search(&self, param: SearchParam) -> Result<super::SearchResult, Error> {
        todo!()
    }

    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        let id = get_id(param.document)?;

        self.identifier_manager.insert(identifier::InsertParam {
            collection: param.collection,
            id: &id,
            document: param.document,
        })

        // TODO: should parallelize index insertion & wait until all index inserted before return (blocking on index unready)
    }

    fn validate_field(&self, field_type: &ValueType, index_type: IndexType) -> Result<(), Error> {
        let index_types_opt= self.allowed_index_types.get(field_type);
        let Some(index_types) = index_types_opt else {
            return Err(Error::new(errcode::PARSE_ERROR, "unknown index type"));
        };

        if !index_types.contains(index_type) {
            return Err(Error::new(errcode::PARSE_ERROR, "index type is unsupported for field type"));
        }

        Ok(())
    }
}
