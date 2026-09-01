use std::sync::Arc;

use crate::{translog::{InsertPortParam, service_param_result::InsertParam}, utils::vixerr::Error};

pub trait TranslogService: Send + Sync {
    fn insert(&self, param: InsertParam) -> Result<(), Error>;
}

pub trait TranslogPort: Send + Sync {
    fn insert(&self, param: InsertPortParam) -> Result<(), Error>;
}

pub struct TranslogServiceImpl {
    adapter: Arc<dyn TranslogPort>,
}

impl TranslogService for TranslogServiceImpl {
    fn insert(&self, param: InsertParam) -> Result<(), Error> {
        self.adapter.insert(param)
    }
}
