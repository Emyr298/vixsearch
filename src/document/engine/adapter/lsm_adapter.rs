use std::sync::Arc;

use fastbloom::BloomFilter;
use uuid::Uuid;

use crate::{document::engine::{LSMPort, lsm_port_param_result::GetMetadataPortResult, lsm_state::CollectionBuffer}, errcode::FATAL_ERROR, storage::{BlockStorage, BlockWriter}, utils::vixerr::Error};

pub const BLOCK_HEADER_SIZE: usize = 20;
pub const METADATA_HEADER_SIZE: usize = 16;
pub const FOOTER_SIZE: usize = 16;
pub const BLOCK_MAGIC: &[u8] = b"BLCK";
pub const FOOTER_MAGIC: &[u8] = b"FOOT";
pub const METADATA_MAGIC: &[u8] = b"META";


pub struct LSMAdapter {
    storage: Arc<dyn BlockStorage>,
    min_document_content_size: usize,
    false_positive_probability: f64,
}

impl LSMPort for LSMAdapter {
    fn get_metadata(&self, segment_id: &str) -> Result<GetMetadataPortResult, Error> {
        let name = self.get_name(segment_id);
        let metadata_offset = self.get_metadata_offset(&name)?;

        let metadata_header = self.storage.read(&name, metadata_offset, METADATA_HEADER_SIZE as u64)?;

        let magic = &metadata_header[0..4];
        if magic != METADATA_MAGIC {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: bad metadata magic"))
                .throw();
        }

        let expected_hash = u32::from_le_bytes(metadata_header[4..8].try_into().unwrap());
        let metadata_len_buf: [u8; 8] = metadata_header[8..METADATA_HEADER_SIZE].try_into().unwrap();
        let metadata_len = u64::from_le_bytes(metadata_len_buf);

        let metadata_content_offset = metadata_offset + (METADATA_HEADER_SIZE as u64);
        let metadata_content = self.storage.read(&name, metadata_content_offset, metadata_len)?;

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&metadata_len_buf);
        hasher.update(&metadata_content);
        let hash = hasher.finalize();

        if expected_hash != hash {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: metadata hash mismatch"))
                .throw();
        }

        let offsets_len: usize = u64::from_le_bytes(metadata_content[0..8].try_into().unwrap())
            .try_into()
            .unwrap();
        let offsets: Vec<u64> = rmp_serde::from_slice(&metadata_content[8..(8+offsets_len)]).unwrap();

        let filter_hash_cnt_offset = 8 + offsets_len;
        let filter_hash_cnt = u32::from_le_bytes(metadata_content[filter_hash_cnt_offset..(filter_hash_cnt_offset+4)].try_into().unwrap());
        
        let filter_len_offset = filter_hash_cnt_offset + 4;
        let filter_len: usize = u64::from_le_bytes(metadata_content[filter_len_offset..filter_len_offset+8].try_into().unwrap())
            .try_into()
            .unwrap();

        let filter_bytes_offset = filter_len_offset + 8;
        let filter_bytes = &metadata_content[filter_bytes_offset..(filter_bytes_offset + (filter_len as usize))];
        let filter_bits: Vec<u64> = filter_bytes
            .chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let smallest_key_len_offset = filter_bytes_offset + (filter_len as usize);
        let smallest_key_len: usize = u64::from_le_bytes(metadata_content[smallest_key_len_offset..smallest_key_len_offset+8].try_into().unwrap())
            .try_into()
            .unwrap();

        let smallest_key_offset = smallest_key_len_offset + 8;
        let smallest_key = str::from_utf8(&metadata_content[smallest_key_offset..(smallest_key_offset + smallest_key_len)]).unwrap().to_string();

        let biggest_key_len_offset = smallest_key_offset + smallest_key_len;
        let biggest_key_len: usize = u64::from_le_bytes(metadata_content[biggest_key_len_offset..biggest_key_len_offset+8].try_into().unwrap())
            .try_into()
            .unwrap();

        let biggest_key_offset = biggest_key_len_offset + 8;
        let biggest_key = str::from_utf8(&metadata_content[biggest_key_offset..(biggest_key_offset + biggest_key_len)]).unwrap().to_string();

        Ok(GetMetadataPortResult{
            offsets,
            filter_hash_cnt,
            filter_bits,
            smallest_key,
            biggest_key,
        })
    }

    fn get_values_from_block(&self, segment_id: &str, block_offset: u64) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Error> {
        let name = self.get_name(&segment_id);

        let block_header = self.storage.read(&name, block_offset, BLOCK_HEADER_SIZE as u64)?;

        let magic = &block_header[0..4];
        if magic != BLOCK_MAGIC {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: bad block magic"))
                .throw();
        }

        let expected_block_len_hash = u32::from_le_bytes(block_header[4..8].try_into().unwrap());
        let block_len_buf: [u8; 8] = block_header[8..16].try_into().unwrap();
        if expected_block_len_hash != crc32fast::hash(&block_len_buf) {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: block length hash mismatch"))
                .throw();
        }

        let block_len = u64::from_le_bytes(block_len_buf);

        let expected_block_hash = u32::from_le_bytes(block_header[16..BLOCK_HEADER_SIZE].try_into().unwrap());
        let block = self.storage.read(&name, block_offset + (BLOCK_HEADER_SIZE as u64), block_len)?;
        if expected_block_hash != crc32fast::hash(&block) {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: block hash mismatch"))
                .throw();
        }

        
















        todo!()
    }

    fn flush_segment(&self, collection_buffer: Arc<CollectionBuffer>) -> Result<(), Error> {
        let segment_id = Uuid::new_v4().to_string();
        let name = self.get_name(&segment_id);

        if collection_buffer.map.len() == 0 {
            return Ok(());
        }

        let mut kv_pairs: Vec<_> = collection_buffer.map
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();

        kv_pairs.sort_by(|a, b| a.0.cmp(&b.0));

        let writer = self.storage.writer(&name)?;

        let (metadata_offset, block_offsets) = self.write_documents(&writer, &kv_pairs)?;
        let footer_offset = self.write_metadata(&writer, metadata_offset, &kv_pairs, block_offsets)?;
        self.write_footer(&writer, footer_offset, metadata_offset)?;

        writer.commit()?;

        Ok(())
    }
}

impl LSMAdapter {
    fn write_documents(&self, writer: &Box<dyn BlockWriter>, kv_pairs: &Vec<(Vec<u8>, Vec<u8>)>) -> Result<(u64, Vec<u64>), Error> {
        let mut block_offsets: Vec<u64> = Vec::new();
        let mut block_content: Vec<u8> = Vec::new();
        let mut next_offset: u64 = 0;
        for (key, value) in kv_pairs.iter() {
            if block_content.len() >= self.min_document_content_size {
                block_offsets.push(next_offset);
                next_offset = self.write_document_block(writer, next_offset, &block_content)?;
                block_content = Vec::new();
            }

            // <KEY LEN 8 byte><VALUE LEN 8 byte><KEY><VALUE>
            block_content.extend((key.len() as u64).to_le_bytes());
            block_content.extend((value.len() as u64).to_le_bytes());
            block_content.extend(key);
            block_content.extend(value);
        }

        if block_content.len() > 0 {
            block_offsets.push(next_offset);
            next_offset = self.write_document_block(writer, next_offset, &block_content)?;
        }

        Ok((next_offset, block_offsets))
    }

    // <BLCK 4 byte><CRC CONTENT LEN 4 byte><CONTENT LEN 8 byte><CRC CONTENT 4 byte><CONTENT>
    fn write_document_block(&self, writer: &Box<dyn BlockWriter>, offset: u64, content_bytes: &[u8]) -> Result<u64, Error> {
        let content_len = (content_bytes.len() as u64).to_le_bytes();

        let content_checksum = crc32fast::hash(&content_bytes);
        let content_len_checksum = crc32fast::hash(&content_len);

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(BLOCK_MAGIC);
        buf.extend_from_slice(&content_len_checksum.to_le_bytes());
        buf.extend_from_slice(content_len.as_slice());
        buf.extend_from_slice(&content_checksum.to_le_bytes());
        buf.extend_from_slice(&content_bytes);

        writer.write(offset, &buf)?;

        let next_offset = offset + (buf.len() as u64);
        Ok(next_offset)
    }

    // <META 4 byte><CRC 4 byte><METADATA LEN 8 byte><OFFSETS LEN 8 byte><OFFSETS>
    // <BLOOMFILTER HASH CNT 4 byte><BLOOMFILTER BITS LEN 8 byte><BF BITS>
    // <SMALLEST KEY LEN><SMALLEST KEY><BIGGEST KEY LEN><BIGGEST KEY>
    fn write_metadata(&self, writer: &Box<dyn BlockWriter>, offset: u64, kv_pairs: &Vec<(Vec<u8>, Vec<u8>)>, block_offsets: Vec<u64>) -> Result<u64, Error> {
        let offsets_bytes = rmp_serde::to_vec(&block_offsets).unwrap();
        let offsets_len = (offsets_bytes.len() as u64).to_le_bytes();

        let keys: Vec<Vec<u8>> = kv_pairs.iter()
            .map(|kv| kv.0.clone())
            .collect();

        let filter = BloomFilter::with_false_pos(self.false_positive_probability)
            .items(keys.iter());
        let filter_hashes = filter.num_hashes().to_le_bytes();
        let filter_bytes: Vec<u8> = filter.iter()
            .flat_map(|block| block.to_le_bytes()) // u64
            .collect();
        let filter_len = (filter_bytes.len() as u64).to_le_bytes();

        let Some(smallest_key) = keys.first() else {
            return Error::code(FATAL_ERROR)
                .message("invalid first key")
                .throw();
        };
        let smallest_key_len = (smallest_key.len() as u64).to_le_bytes();

        let Some(biggest_key) = keys.last() else {
            return Error::code(FATAL_ERROR)
                .message("invalid last key")
                .throw();
        };
        let biggest_key_len = (biggest_key.len() as u64).to_le_bytes();

        let content_size = offsets_len.len()
            + offsets_bytes.len()
            + filter_hashes.len()
            + filter_len.len()
            + filter_bytes.len()
            + smallest_key_len.len()
            + smallest_key.len()
            + biggest_key_len.len()
            + biggest_key.len();
        let content_len = (content_size as u64).to_le_bytes();

        let content_bytes: Vec<u8> = [
            content_len.as_slice(),
            offsets_len.as_slice(),
            offsets_bytes.as_slice(),
            filter_hashes.as_slice(),
            filter_len.as_slice(),
            filter_bytes.as_slice(),
            smallest_key_len.as_slice(),
            smallest_key,
            biggest_key_len.as_slice(),
            biggest_key,
        ].concat();

        let checksum = crc32fast::hash(&content_bytes);

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(METADATA_MAGIC);
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.extend_from_slice(&content_bytes);

        writer.write(offset, &buf)?;

        let next_offset = offset + (buf.len() as u64);
        Ok(next_offset)
    }

    // <FOOT 4 byte><CRC 4 byte><METADATA OFFSET 8 byte>
    fn write_footer(&self, writer: &Box<dyn BlockWriter>, offset: u64, metadata_offset: u64) -> Result<(), Error> {
        let metadata_offset_bytes = metadata_offset.to_le_bytes();
        let checksum = crc32fast::hash(&metadata_offset_bytes);
        
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(FOOTER_MAGIC);
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.extend_from_slice(&metadata_offset_bytes);

        writer.write(offset, &buf)?;

        Ok(())
    }
}

impl LSMAdapter {
    fn get_name(&self, segment_id: &str) -> String {
        format!("log_{}", segment_id)
    }

    fn get_metadata_offset(&self, name: &str) -> Result<u64, Error> {
        let size = self.storage.size(name)?;

        let Some(footer_offset) = size.checked_sub(FOOTER_SIZE as u64) else {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: size is under footer size {}", FOOTER_SIZE))
                .throw()
        };

        let footer = self.storage.read(name, footer_offset, FOOTER_SIZE as u64)?;

        let magic = &footer[0..4];
        if magic != FOOTER_MAGIC {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: bad footer magic"))
                .throw();
        }

        let expected_hash = u32::from_le_bytes(footer[4..8].try_into().unwrap());
        let metadata_offset: [u8; 8] = footer[8..FOOTER_SIZE].try_into().unwrap();
        let hash = crc32fast::hash(&metadata_offset);
        if expected_hash != hash {
            return Error::code(FATAL_ERROR)
                .message(format!("invalid block format: footer hash mismatch"))
                .throw();
        }

        Ok(u64::from_le_bytes(metadata_offset))
    }
}
