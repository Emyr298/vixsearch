use std::{fs::File, io::ErrorKind, os::unix::fs::FileExt, sync::Arc};

use crate::{errcode::FATAL_ERROR, storage::{StorageReader, error::OUT_OF_RANGE}, utils::vixerr::Error};

pub struct FileStorageReader {
    file: Arc<File>,
}

impl FileStorageReader {
    pub fn new(file: Arc<File>) -> Box<dyn StorageReader> {
        Box::new(Self {
            file,
        })
    }
}

impl StorageReader for FileStorageReader {
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; size as usize];
        
        match self.file.read_exact_at(&mut buf, offset) {
            Ok(()) => Ok(buf),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => Error::code(OUT_OF_RANGE)
                .message("read beyond end of file")
                .wrap(e)
                .throw(),
            Err(e) => Error::code(FATAL_ERROR)
                .message("failed to read from file")
                .wrap(e)
                .throw(),
        }
    }
    
    fn read_all(&self) -> Result<Vec<u8>, Error> {
        let byte_size = self.byte_size()?;
        self.read(0, byte_size)
    }
    
    fn byte_size(&self) -> Result<u64, Error> {
        match self.file.metadata() {
            Ok(m) => return Ok(m.len()),
            Err(err) => return Error::code(FATAL_ERROR)
                .message("failed to open file")
                .wrap(err)
                .throw(),
        }
    }
}
