use crate::utils::vixerr::Error;

pub trait BlockStorage: Send + Sync {
    fn read(&self, name: &str, offset: u64, size: u64) -> Result<Vec<u8>, Error>;
    fn write(&self, name: &str, offset: u64, data: &[u8]) -> Result<(), Error>;
    fn commit(&self, name: &str) -> Result<(), Error>;
}
