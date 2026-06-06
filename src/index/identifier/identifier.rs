use crate::{index::layered::{self, InsertParam}, utils::vixerr::Error};



pub trait Manager {
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
}

pub struct SearchParam {
    pub id: String,
}

pub struct SearchResult {
    pub offset: String,
}

struct ManagerImpl {
    base: layered::Base<SearchParam, SearchResult>,
}

impl Manager for ManagerImpl {
    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        return self.base.insert(param);
    }
}

