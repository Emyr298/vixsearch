use std::{fs::{File, OpenOptions}, io::{Seek, Write}, path::Path};

use crate::{errcode, shared::Document, utils::vixerr::Error};

pub struct Writer {
    base_dir: String,
    segment_id: String,
    page_size: usize,
}

impl Writer {
    pub fn write(&self, id: Vec<&str>, doc: Vec<Document>) -> Result<(), Error> {
        let doc_path = Path::new(&self.base_dir).join(format!("doc_{}", &self.segment_id));
        let mut doc_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(doc_path)
            .map_err(|e| Error::code(errcode::FATAL_ERROR).wrap(e))?;

        let log_path = Path::new(&self.base_dir).join(format!("log_{}", &self.segment_id));
        let mut log_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_path)
            .map_err(|e| Error::code(errcode::FATAL_ERROR).wrap(e))?;

        // header
        self.write_header(&mut log_file)?;

        // documents


        

        

        Ok(())
    }

    // TODO: bloomfilter
    fn write_header(&self, log_file: &mut File) -> Result<(), Error> {
        Ok(())
    }

    fn write_block(&self, ids: Vec<&str>, docs: Vec<Document>, log_file: &mut File, doc_file: &mut File) -> Result<(), Error> {
        let mut order: Vec<u64> = Vec::new();

        let mut block = Block::new(self.page_size.clone());
        for (id, doc) in ids.iter().zip(docs.iter()) {
            let doc_json = serde_json::to_string(doc)
                .map_err(|e| Error::code(errcode::FATAL_ERROR).message("failed to convert doc into json").wrap(e))?;

            let id_bytes = id.as_bytes();
            let doc_json_bytes = doc_json.as_bytes();
            let total_len = id_bytes.len() + 1 + doc_json_bytes.len();

            if total_len > block.size_left() {
                if block.is_empty() {
                    // put partial into current page and continue to new page
                }

                // flush current block
                let offset = log_file
                    .stream_position()
                    .map_err(|e| Error::code(errcode::FATAL_ERROR).message("invalid offset").wrap(e))?;
                order.push(offset);

                let buf = block.write()?;
                log_file.write_all(&buf);

                block = Block::new(self.page_size.clone());
            }

            block.extend(id_bytes);
            block.push(b':');
            block.extend(doc_json_bytes);
        }

        Ok(())
    }
}

struct Block {
    total_size: usize,
    content_size: usize,
    content_buffer: Vec<u8>,
    has_next: bool,
}

impl Block {
    fn new(size: usize) -> Self {
        Self {
            total_size: size,
            content_size: size - 9,
            content_buffer: Vec::new(),
            has_next: false,
        }
    }

    fn extend(&mut self, bytes: &[u8]) {
        self.content_buffer.extend_from_slice(bytes);
    }

    fn push(&mut self, byte: u8) {
        self.content_buffer.push(byte);
    }

    fn is_empty(&self) -> bool {
        self.content_buffer.len() == 0
    }

    fn size_left(&self) -> usize {
        self.content_size - self.content_buffer.len()
    }

    fn mark_next(&mut self) {
        self.has_next = true;
    }

    // <PAGE 4 byte><CRC 4 byte><HASNEXT 1 byte>
    fn write(self) -> Result<Vec<u8>, Error> {
        if self.size_left() < 0 {
            return Err(Error::code(errcode::FATAL_ERROR).message("content exceeds block size"));
        }

        let mut buf: Vec<u8> = Vec::new();

        let checksum = crc32fast::hash(&self.content_buffer);

        buf.extend_from_slice(b"PAGE");
        buf.extend_from_slice(&checksum.to_le_bytes());
        buf.push(u8::from(self.has_next));
        buf.extend_from_slice(&self.content_buffer);

        buf
    }
}
