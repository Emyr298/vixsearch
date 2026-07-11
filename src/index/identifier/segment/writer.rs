use std::{fs::{File, OpenOptions}, io::{Seek, Write}, path::Path};

use fastbloom::BloomFilter;

use crate::{errcode, index::identifier::segment::constants::{BLOCK_HEADER_SIZE, FOOTER_MAGIC, METADATA_MAGIC}, shared::Document, utils::vixerr::Error};

pub struct Writer {
    base_dir: String,
    segment_id: String,
    min_document_block_size: usize,
    min_document_content_size: usize,
    false_positive_probability: f64,
}

impl Writer {
    pub fn new(base_dir: String, segment_id: String, min_document_block_size: usize, false_positive_probability: f64) -> Self {
        Self {
            base_dir,
            segment_id,
            min_document_block_size,
            min_document_content_size: min_document_block_size - BLOCK_HEADER_SIZE,
            false_positive_probability
        }
    }

    pub fn write(&self, ids: Vec<&str>, docs: Vec<Document>) -> Result<(), Error> {
        if ids.len() == 0 || ids.len() != docs.len() {
            return Error::code(errcode::FATAL_ERROR)
                .message("invalid ids and docs")
                .throw();
        }

        let file_path = Path::new(&self.base_dir).join(format!("log_{}", &self.segment_id));
        let file_result = OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(file_path);

        let mut file = match file_result {
            Ok(val) => val,
            Err(err) => return Error::code(errcode::FATAL_ERROR)
                .message("failed to open file")
                .wrap(err)
                .throw(),
        };

        let block_offsets = self.write_documents(&mut file, &ids, &docs)?;
        let metadata_offset = self.write_metadata(&mut file, ids, block_offsets)?;
        self.write_footer(&mut file, metadata_offset)?;

        if let Err(err) = file.sync_all() {
            return Error::code(errcode::FATAL_ERROR)
                .message("failed to flush file")
                .wrap(err)
                .throw();
        }

        Ok(())
    }

    fn write_documents(&self, file: &mut File, ids: &Vec<&str>, docs: &Vec<Document>) -> Result<Vec<u64>, Error> {
        let mut block_offsets: Vec<u64> = Vec::new();
        let mut block_content: Vec<u8> = Vec::new();
        for (id, doc) in ids.iter().zip(docs.iter()) {
            let doc_json = match serde_json::to_string(doc) {
                Ok(v) => v,
                Err(err) => return Error::code(errcode::FATAL_ERROR)
                    .message("failed to convert doc into json")
                    .wrap(err)
                    .throw(),
            };

            let id_bytes = id.as_bytes();
            let doc_json_bytes = doc_json.as_bytes();

            if block_content.len() >= self.min_document_content_size {
                let offset = self.write_document_block(file, block_content.as_slice())?;
                block_offsets.push(offset);
                block_content = Vec::new();
            }

            let total_len = id_bytes.len() + 1 + doc_json_bytes.len();
            block_content.extend((total_len as u64).to_le_bytes());
            block_content.extend(id_bytes);
            block_content.push(b':');
            block_content.extend(doc_json_bytes);
        }

        if block_content.len() > 0 {
            let offset = self.write_document_block(file, block_content.as_slice())?;
            block_offsets.push(offset);
        }

        Ok(block_offsets)
    }

    // <BLCK 4 byte><CRC 4 byte><CONTENT LEN 8 byte><CONTENT>
    fn write_document_block(&self, file: &mut File, content_bytes: &[u8]) -> Result<u64, Error> {
        let checksum = crc32fast::hash(&content_bytes);
        let content_len = (content_bytes.len() as u64).to_le_bytes();

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"BLCK");
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.extend_from_slice(content_len.as_slice());
        buf.extend_from_slice(&content_bytes);

        let offset = match file.stream_position() {
            Ok(val) => val,
            Err(err) => return Error::code(errcode::FATAL_ERROR)
                .message("invalid offset")
                .wrap(err)
                .throw(),
        };

        if let Err(err) = file.write_all(&buf) {
            return Error::code(errcode::FATAL_ERROR)
                .message("failed to write_all")
                .wrap(err)
                .throw();
        }

        Ok(offset)
    }

    // <META 4 byte><CRC 4 byte><METADATA LEN 8 byte><OFFSETS LEN 8 byte><OFFSETS>
    // <BLOOMFILTER HASH CNT 4 byte><BLOOMFILTER BITS LEN 8 byte><BF BITS>
    // <SMALLEST KEY LEN><SMALLEST KEY><BIGGEST KEY LEN><BIGGEST KEY>
    fn write_metadata(&self, file: &mut File, ids: Vec<&str>, block_offsets: Vec<u64>) -> Result<u64, Error> {
        let offsets_bytes = rmp_serde::to_vec(&block_offsets).unwrap();
        let offsets_len = (offsets_bytes.len() as u64).to_le_bytes();

        let filter = BloomFilter::with_false_pos(self.false_positive_probability)
            .items(ids.iter());

        let filter_hashes = filter.num_hashes().to_le_bytes();

        let filter_bytes: Vec<u8> = filter
            .iter()
            .flat_map(|block| block.to_le_bytes()) // u64
            .collect();
        let filter_len = (filter_bytes.len() as u64).to_le_bytes();

        let Some(smallest_key) = ids.first() else {
            return Error::code(errcode::FATAL_ERROR)
                .message("invalid first key")
                .throw();
        };
        let smallest_key_len = (smallest_key.len() as u64).to_le_bytes();

        let Some(biggest_key) = ids.last() else {
            return Error::code(errcode::FATAL_ERROR)
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
            smallest_key.as_bytes(),
            biggest_key_len.as_slice(),
            biggest_key.as_bytes(),
        ].concat();

        let checksum = crc32fast::hash(&content_bytes);

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(METADATA_MAGIC);
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.extend_from_slice(&content_bytes);

        let offset = match file.stream_position() {
            Ok(val) => val,
            Err(err) => return Error::code(errcode::FATAL_ERROR)
                .message("invalid offset")
                .wrap(err)
                .throw(),
        };

        if let Err(err) = file.write_all(&buf) {
            return Error::code(errcode::FATAL_ERROR)
                .message("failed to write_all")
                .wrap(err)
                .throw();
        }

        Ok(offset)
    }

    // <FOOT 4 byte><CRC 4 byte><METADATA OFFSET 8 byte>
    fn write_footer(&self, file: &mut File, metadata_offset: u64) -> Result<(), Error> {
        let offset_bytes = metadata_offset.to_le_bytes();
        let checksum = crc32fast::hash(&offset_bytes);
        
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(FOOTER_MAGIC);
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.extend_from_slice(&offset_bytes);

        if let Err(err) = file.write_all(&buf) {
            return Error::code(errcode::FATAL_ERROR)
                .message("failed to write_all")
                .wrap(err)
                .throw();
        }

        Ok(())
    }
}
