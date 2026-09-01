use crate::utils::vixerr::Error;

#[deprecated]
pub trait LegacyStorage: Send + Sync {
    fn get_all_name_sorted(&self, store: &str) -> Result<Vec<String>, Error>;
    fn open(&self, store: &str, name: &str) -> Result<Box<dyn LegacyStorageAccessor>, Error>;
    fn close(&self, store: &str, name: &str) -> Result<(), Error>;
}

#[deprecated]
pub trait LegacyStorageAccessor: Send + Sync {
    fn size(&self) -> Result<u64, Error>;
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error>;
    fn flush(&self) -> Result<(), Error>;
}

pub trait Storage: Send + Sync {
    fn get_all_name_sorted(&self, store: &str) -> Result<Vec<String>, Error>;
    fn get_append_only_writer(&self, store: &str, name: &str, last_offset: u64) -> Result<Box<dyn AppendOnlyStorageWriter>, Error>;
    fn get_write_once_writer(&self, store: &str, name: &str) -> Result<Box<dyn WriteOnceStorageWriter>, Error>;
    fn get_reader(&self, store: &str, name: &str) -> Result<Box<dyn StorageReader>, Error>;
    fn close(&self, store: &str, name: &str) -> Result<(), Error>;
}

pub trait StorageReader: Send + Sync {
    fn read_all(&self) -> Result<Vec<u8>, Error>;
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
}

pub trait AppendOnlyStorageWriter: Send + Sync {
    fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    fn flush(&self) -> Result<(), Error>;
}

pub trait WriteOnceStorageWriter: Send + Sync {
    fn write(&mut self, data: &[u8]) -> Result<(), Error>;
    fn commit(self: Box<Self>) -> Result<(), Error>;
}
