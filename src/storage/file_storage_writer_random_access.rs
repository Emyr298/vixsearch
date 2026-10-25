use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::FileExt,
    path::PathBuf,
    sync::Arc,
};

use crate::{errcode::FATAL_ERROR, storage::RandomAccessStorageWriter, utils::vixerr::Error};

pub struct RandomAccessFileStorageWriter {
    file: Arc<File>,
}

impl RandomAccessFileStorageWriter {
    pub fn new(file: Arc<File>) -> Arc<dyn RandomAccessStorageWriter> {
        Arc::new(Self { file })
    }
}

impl RandomAccessStorageWriter for RandomAccessFileStorageWriter {
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error> {
        if let Err(e) = self.file.write_all_at(data, offset) {
            return Error::code(FATAL_ERROR)
                .message("failed to write to file")
                .wrap(e)
                .throw();
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), Error> {
        if let Err(e) = self.file.sync_all() {
            return Error::code(FATAL_ERROR)
                .message("failed to flush file")
                .wrap(e)
                .throw();
        };

        Ok(())
    }
}
