use std::{fs::{File, OpenOptions}, io::{Seek, Write}, path::Path};

use crate::{errcode, shared::Document, utils::vixerr::Error};

const page_header_size: usize = 9;

pub struct Writer {
    base_dir: String,
    segment_id: String,
    min_document_block_size: usize,
    min_document_content_size: usize,
}

impl Writer {
    pub fn new(base_dir: String, segment_id: String, min_document_block_size: usize) -> Self {
        Self {
            base_dir,
            segment_id,
            min_document_block_size,
            min_document_content_size: min_document_block_size - page_header_size,
        }
    }

    pub fn write(&self, ids: Vec<&str>, docs: Vec<Document>) -> Result<(), Error> {
        let file_path = Path::new(&self.base_dir).join(format!("log_{}", &self.segment_id));
        let file_result = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path);

        let mut file = match file_result {
            Ok(val) => val,
            Err(err) => return Error::code(errcode::FATAL_ERROR)
                .message("failed to open file")
                .wrap(err)
                .throw(),
        };

        let block_offsets = self.write_documents(&mut file, &ids, &docs)?;
        self.write_footer(&mut file, block_offsets)?;

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

    // <BLCK 4 byte><CRC 4 byte><CONTENT>
    fn write_document_block(&self, file: &mut File, content_bytes: &[u8]) -> Result<u64, Error> {
        let checksum = crc32fast::hash(&content_bytes);

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"BLCK");
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.extend_from_slice(&content_bytes);

        if let Err(err) = file.write_all(&buf) {
            return Error::code(errcode::FATAL_ERROR)
                .message("failed to write_all")
                .wrap(err)
                .throw();
        }

        let offset = match file.stream_position() {
            Ok(val) => val,
            Err(err) => return Error::code(errcode::FATAL_ERROR)
                .message("invalid offset")
                .wrap(err)
                .throw(),
        };

        Ok(offset)
    }

    // TODO: bloomfilter
    // <FOOT 4 byte><CRC 4 byte><OFFSETS LEN><OFFSETS><BLOOMFILTER LEN><BLOOMFILTER>
    fn write_footer(&self, file: &mut File, block_offsets: Vec<u64>) -> Result<(), Error> {
        let offsets_bytes = rmp_serde::to_vec(&block_offsets).unwrap();
        let checksum = crc32fast::hash(&offsets_bytes);

        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(b"FOOT");
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.extend((offsets_bytes.len() as u64).to_le_bytes());
        buf.extend_from_slice(&offsets_bytes);

        if let Err(err) = file.write_all(&buf) {
            return Error::code(errcode::FATAL_ERROR)
                .message("failed to write_all")
                .wrap(err)
                .throw();
        }

        Ok(())
    }
}
