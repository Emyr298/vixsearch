use std::{fs::File, os::unix::fs::FileExt, sync::Arc};

use crate::{errcode::FATAL_ERROR, storage::StorageAccessor, utils::vixerr::Error};

pub struct FileStorageAccessor {
    file: Arc<File>,
}

impl FileStorageAccessor {
    pub fn new(file: Arc<File>) -> Box<dyn StorageAccessor> {
        Box::new(Self {
            file,
        })
    }
}

impl StorageAccessor for FileStorageAccessor {
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

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error> {
        if let Err(e) = self.file.write_at(data, offset) {
            return Error::code(FATAL_ERROR)
                .message("failed to write to file")
                .wrap(e)
                .throw();
        };

        Ok(())
    }

    fn flush(&self) -> Result<(), Error> {
        if let Err(e) = self.file.sync_all() {
            return Error::code(FATAL_ERROR)
                .message(format!("failed to flush file"))
                .wrap(e)
                .throw();
        };

        Ok(())
    }
}
