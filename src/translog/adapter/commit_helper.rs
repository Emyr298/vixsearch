use std::sync::Arc;

use crate::{errcode::PARSE_ERROR, storage::Storage, utils::vixerr::Error};
use crc32fast::Hasher;
use deku::{DekuContainerRead, DekuContainerWrite, DekuRead, DekuWrite};
use rmp_serde::to_vec;
use serde::{Deserialize, Serialize};

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

pub trait CommitStorage<T>: Send + Sync
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn submit(&self, store: &str, commit_number: u64, data: T) -> Result<(), Error>;
    fn get_last_commit(&self, store: &str) -> Result<GetLastCommitResult<T>, Error>;
}

pub struct CommitStorageImpl {
    storage: Arc<dyn Storage>,
}

impl CommitStorageImpl {
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    fn get_commit<T>(&self, commit_store: &str, commit_name: &str) -> Result<T, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let reader = self
            .storage
            .get_or_create_reader(commit_store, commit_name)?;
        let commit_buf = reader.read_all()?;

        let commit_block = CommitBlock::from_buf(commit_buf)?;
        commit_block.decode()
    }
}

impl<T> CommitStorage<T> for CommitStorageImpl
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn submit(&self, store: &str, commit_number: u64, data: T) -> Result<(), Error> {
        let commit_block = CommitBlock::encode(data);
        let commit_buf = commit_block.to_buf();

        let name = format!("{}_{:016x}", NAME_PREFIX, commit_number);
        let mut writer = self.storage.create_write_once_writer(store, &name)?;

        writer.write(&commit_buf)?;
        writer.commit()?;

        Ok(())
    }

    fn get_last_commit(&self, store: &str) -> Result<GetLastCommitResult<T>, Error> {
        let names = self.storage.get_all_name_sorted(&store)?;
        let next_commit_number = next_commit_number(&names);

        let prefix = format!("{}_", NAME_PREFIX);
        for commit_name in names.into_iter().filter(|n| n.starts_with(&prefix)).rev() {
            match self.get_commit(&store, &commit_name) {
                Ok(c) => return Ok(GetLastCommitResult::new(next_commit_number, Some(c))),
                Err(e) if e.code != PARSE_ERROR => return Err(e),
                Err(_) => continue,
            }
        }

        Ok(GetLastCommitResult::new(next_commit_number, None))
    }
}

fn next_commit_number(names_asc: &[String]) -> u64 {
    let prefix = format!("{}_", NAME_PREFIX);

    for name in names_asc.iter().rev() {
        let Some(hex) = name.strip_prefix(&prefix) else {
            continue;
        };

        if hex.len() == 16 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let num = u64::from_str_radix(hex, 16).unwrap();
            return num.checked_add(1).unwrap();
        }
    }

    1
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct CommitBlock {
    magic: [u8; 4],
    hash: [u8; 4],
    data_len: u64,
    #[deku(count = "data_len")]
    data: Vec<u8>,
}

impl CommitBlock {
    pub fn encode<T>(data: T) -> Self
    where
        T: Serialize,
    {
        let data_buf = rmp_serde::to_vec(&data).unwrap();

        CommitBlock {
            magic: *COMMIT_MAGIC,
            hash: crc32fast::hash(&data_buf).to_le_bytes(),
            data_len: (data_buf.len() as u64),
            data: data_buf,
        }
    }

    pub fn decode<T>(&self) -> Result<T, Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        let commit: T = match rmp_serde::from_slice(&self.data) {
            Ok(c) => c,
            Err(e) => {
                return Error::code(PARSE_ERROR)
                    .message("failed to decode commit buffer")
                    .wrap(e)
                    .throw();
            }
        };

        Ok(commit)
    }

    pub fn from_buf(buf: Vec<u8>) -> Result<Self, Error> {
        let (_, block) = match CommitBlock::from_bytes((&buf, 0)) {
            Ok(b) => b,
            Err(e) => {
                return Error::code(PARSE_ERROR)
                    .message("failed to decode commit buffer")
                    .wrap(e)
                    .throw();
            }
        };

        let expected_hash = u32::from_le_bytes(block.hash.try_into().unwrap());
        let actual_hash = crc32fast::hash(&block.data);
        if expected_hash != actual_hash {
            return Error::code(PARSE_ERROR)
                .message("invalid commit hash")
                .throw();
        }

        Ok(block)
    }

    pub fn to_buf(self) -> Vec<u8> {
        self.to_bytes().unwrap()
    }
}
