use crate::utils::vixerr::Error;

pub trait Storage: Send + Sync {
    fn get_all_name(&self, store: &str) -> Result<Vec<String>, Error>;
    fn open(&self, store: &str, name: &str) -> Result<Box<dyn StorageAccessor>, Error>;
    fn close(&self, store: &str, name: &str) -> Result<(), Error>;
}

pub trait StorageAccessor: Send + Sync {
    fn size(&self) -> Result<u64, Error>;
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error>;
    fn flush(&self) -> Result<(), Error>;
}
