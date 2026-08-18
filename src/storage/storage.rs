use crate::utils::vixerr::Error;

pub trait Storage: Send + Sync {
    fn get_all_name_sorted(&self, store: &str) -> Result<Vec<String>, Error>;
    fn open(&self, store: &str, name: &str) -> Result<Box<dyn StorageAccessor>, Error>;
    fn close(&self, store: &str, name: &str) -> Result<(), Error>;
}

pub trait StorageAccessor: Send + Sync {
    fn size(&self) -> Result<u64, Error>;
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error>;
    fn flush(&self) -> Result<(), Error>;
}

pub trait WormStorage: Send + Sync {
    fn get_all_commited_name_sorted(&self, store: &str) -> Result<Vec<String>, Error>;
    fn get_writer(&self, store: &str, name: &str) -> Result<Box<dyn WormWriter>, Error>;
    fn get_reader(&self, store: &str, name: &str) -> Result<Box<dyn WormReader>, Error>;
    fn close(&self, store: &str, name: &str) -> Result<(), Error>;
}

pub trait WormReader: Send + Sync {
    fn read_all(&self) -> Result<Vec<u8>, Error>;
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
}

pub trait WormWriter: Send + Sync {
    fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    fn commit(&self) -> Result<(), Error>;
}
