use crate::utils::vixerr::Error;

pub trait Pool: Send + Sync {
    fn submit(&self, func: Box<dyn FnOnce() + Send + 'static>) -> Result<(), Error>;
}
