use crate::utils::vixerr::Error;

pub trait Loader<T>: Send + Sync {
    fn load(&self) -> Result<T, Error>;
}
