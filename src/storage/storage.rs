use crate::utils::vixerr::Error;

pub trait BlockStorage: Send + Sync {
    fn size(&self, name: &str) -> Result<u64, Error>;
    fn read(&self, name: &str, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
    fn writer(&self, name: &str) -> Result<Box<dyn BlockWriter>, Error>;
}

pub trait BlockWriter: Send + Sync {
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error>;
    fn commit(self: Box<Self>) -> Result<(), Error>;
}
