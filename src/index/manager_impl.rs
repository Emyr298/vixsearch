use std::{collections::{HashMap, HashSet}, sync::Arc};

use dashmap::{DashMap};

use crate::{errcode, index::{IndexType, InsertParam, SearchParam, field_key, manager::{Manager, TypeManager}, param_result::{CreateParam, DeleteParam}}, shared::ValueType, utils::vixerr::Error};

struct ManagerImpl {
    allowed_index_types: Arc<HashMap<ValueType, HashSet<IndexType>>>,
    type_managers: DashMap<IndexType, Arc<dyn TypeManager>>,
    index_map: DashMap<String, IndexType>,
}

impl ManagerImpl {
    fn new(
        allowed_index_types: HashMap<ValueType, HashSet<IndexType>>,
        type_managers: Vec<(IndexType, Arc<dyn TypeManager>)>
    ) -> Self {
        ManagerImpl {
            allowed_index_types: Arc::new(allowed_index_types),
            type_managers: type_managers.into_iter().collect(),
            index_map: DashMap::new(),
        }
    }
}

impl Manager for ManagerImpl {
    // TODO: fail from this must be panic from orchestrator
    fn create(&self, param: CreateParam) -> Result<(), Error> {
        let key = field_key(&param.collection, &param.field);
        self.index_map.insert(key, param.index_type.clone());

        let type_manager = self.type_managers
            .get(&param.index_type)
            .ok_or_else(|| Error::new(errcode::FATAL_ERROR, "index not found"))?;

        type_manager.create(param.into())
    }
    
    // TODO: fail from this must be panic from orchestrator
    fn delete(&self, param: DeleteParam) -> Result<(), Error> {
        let key = field_key(&param.collection, &param.field);
        let index_type_opt = self.index_map.remove(&key).map(|(_, v)| v);
        let Some(index_type) = index_type_opt else {
            return Ok(());
        };

        let type_manager_opt = self.type_managers
            .get(&index_type);
        let Some(type_manager) = type_manager_opt else {
            return Ok(());
        };

        type_manager.delete(param)
    }

    fn search(&self, param: SearchParam) -> Result<super::SearchResult, Error> {
        todo!()
    }

    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        let key = field_key(&param.collection, &param.field);
        let index_type_opt = self.index_map.get(&key);
        let Some(index_type) = index_type_opt else {
            return Ok(()) // collection is not indexed
        };

        let type_manager_opt = self.type_managers
            .get(*index_type);
        let Some(type_manager) = type_manager_opt else {
            return Err(Error::new(errcode::FATAL_ERROR, "index type manager not found"));
        };

        type_manager.insert(param)
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
