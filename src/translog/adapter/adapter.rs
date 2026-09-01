use std::sync::Arc;

use crate::{storage::Storage, translog::{InsertPortParam, TranslogPort}, utils::vixerr::Error};

pub const STORE: &str = "translog";
pub const NAME: &str = "append.log";

// TODO: make segmented translog with cleanup mechanism
pub struct TranslogAdapter {
    storage: Arc<dyn Storage>,
}

impl TranslogPort for TranslogAdapter {
    fn insert(&self, param: InsertPortParam) -> Result<(), Error> {
        let writer = self.storage.get_write_once_writer(STORE, NAME)?;

        writer.write()

        todo!()
    }
}
