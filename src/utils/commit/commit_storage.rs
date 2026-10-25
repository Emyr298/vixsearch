use crate::utils::vixerr::Error;
use serde::{Deserialize, Serialize};

// TODO: invalid commits should be cleaned at load

pub trait CommitStorage<T>: Send + Sync
where
    T: Serialize + for<'de> Deserialize<'de> + 'static,
{
    fn submit(&self, store: &str, commit_number: u64, data: T) -> Result<(), Error>;
    fn get_last_commit(&self, store: &str) -> Result<GetLastCommitResult<T>, Error>;
}

pub const COMMIT_MAGIC: &[u8; 4] = b"CMMT";
pub const NAME_PREFIX: &str = "commit";

pub struct GetLastCommitResult<T> {
    pub next_number: u64,
    pub commit: Option<T>,
}

impl<T> GetLastCommitResult<T> {
    pub fn new(next_number: u64, commit: Option<T>) -> Self {
        GetLastCommitResult {
            next_number,
            commit,
        }
    }
}
