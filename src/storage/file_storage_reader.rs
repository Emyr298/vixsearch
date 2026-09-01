use std::{fs::File, os::unix::fs::FileExt, sync::Arc};

use crate::{errcode::FATAL_ERROR, storage::StorageReader, utils::vixerr::Error};

pub struct FileStorageReader {
    file: Arc<File>,
}

impl FileStorageReader {
    pub fn new(file: Arc<File>) -> Box<dyn StorageReader> {
        Box::new(Self {
            file,
        })
    }

    fn size(&self) -> Result<u64, Error> {
        let file_metadata = match self.file.metadata() {
            Ok(val) => val,
            Err(err) => return Error::code(FATAL_ERROR)
                .message("failed to open file")
                .wrap(err)
                .throw(),
        };
        
        Ok(file_metadata.len())
    }
}

impl StorageReader for FileStorageReader {
    fn read(&self, offset: u64, size: u64) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; size as usize];
        if let Err(e) = self.file.read_exact_at(&mut buf, offset) {
            return Error::code(FATAL_ERROR)
                .message("failed to read from file")
                .wrap(e)
                .throw();
        };

        Ok(buf)
    }
    
    fn read_all(&self) -> Result<Vec<u8>, Error> {
        let byte_size = self.size()?;
        self.read(0, byte_size)
    }
}
