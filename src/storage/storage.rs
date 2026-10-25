use std::sync::Arc;

use crate::utils::vixerr::Error;

pub trait Storage: Send + Sync {
    fn get_all_name_sorted(&self, store: &str) -> Result<Vec<String>, Error>;
    fn get_or_create_reader(
        &self,
        store: &str,
        name: &str,
    ) -> Result<Box<dyn StorageReader>, Error>;
    fn get_or_create_random_access_writer(
        &self,
        store: &str,
        name: &str,
    ) -> Result<Arc<dyn RandomAccessStorageWriter>, Error>;
    fn create_write_once_writer(
        &self,
        store: &str,
        name: &str,
    ) -> Result<Box<dyn WriteOnceStorageWriter>, Error>;
    fn close(&self, store: &str, name: &str);
}

pub trait StorageReader: Send + Sync {
    fn read_all(&self) -> Result<Vec<u8>, Error>;
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
    fn byte_size(&self) -> Result<u64, Error>;
}

pub trait RandomAccessStorageWriter: Send + Sync {
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error>;
    fn flush(&self) -> Result<(), Error>;
}

pub trait WriteOnceStorageWriter {
    fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    fn commit(self: Box<Self>) -> Result<(), Error>;
}
