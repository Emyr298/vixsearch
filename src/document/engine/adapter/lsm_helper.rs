use serde::{Deserialize, Serialize};

use crate::{document::engine::lsm_port_param_result::{GetAllSegmentByCollectionIDPortResult, GetAllSegmentByCollectionIDPortResultSegment}, errcode::FATAL_ERROR, storage::StorageAccessor, utils::vixerr::Error};

pub const BLOCK_HEADER_SIZE: usize = 20;
pub const METADATA_HEADER_SIZE: usize = 16;
pub const FOOTER_SIZE: usize = 16;
pub const BLOCK_MAGIC: &[u8] = b"BLCK";
pub const FOOTER_MAGIC: &[u8] = b"FOOT";
pub const METADATA_MAGIC: &[u8] = b"META";

pub fn store(collection_id: &str) -> String {
    format!("collection/{}/document", collection_id)
}

pub fn name(segment_id: &str) -> String {
    format!("log_{}", segment_id)
}

pub fn latest_commit_name(names: &[String]) -> Option<String> {
    names.iter()
        .filter(|name| name.starts_with("commit_"))
        .map(|name| name.to_string())
        .max()
}

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub segments: Vec<CommitSegment>,
}

impl Commit {
    pub fn empty() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn get_all_segment_by_collection_id_port_result(self) -> GetAllSegmentByCollectionIDPortResult {
        GetAllSegmentByCollectionIDPortResult {
            segments: self.segments.into_iter()
                .map(|segment| segment.get_all_segment_by_collection_id_port_result_segment())
                .collect()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CommitSegment {
    pub id: String,
    pub level: u32,
}

impl CommitSegment {
    fn get_all_segment_by_collection_id_port_result_segment(self) -> GetAllSegmentByCollectionIDPortResultSegment {
        GetAllSegmentByCollectionIDPortResultSegment {
            id: self.id,
            level: self.level,
        }
    }
}

pub fn get_latest_commit(accessor: Box<dyn StorageAccessor>) -> Result<Commit, Error> {
    let commit_size = accessor.size()?;
    let commit_buf = accessor.read(0, commit_size)?;
    let commit_content_buf: Vec<u8> = commit_buf[4..].to_vec();

    let expected_hash = u32::from_le_bytes(commit_buf[0..4].try_into().unwrap());
    if expected_hash != crc32fast::hash(&commit_content_buf) {
        return Error::code(FATAL_ERROR)
            .message(format!("invalid commit format: hash mismatch"))
            .throw();
    }

    let commit_content: Commit = rmp_serde::from_slice(&commit_content_buf).unwrap();
    Ok(commit_content)
}
