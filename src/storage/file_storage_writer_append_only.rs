use std::{fs::{self, File}, io::Write, os::unix::fs::FileExt, path::PathBuf, sync::{Arc, Mutex}};

use crate::{errcode::FATAL_ERROR, storage::AppendOnlyStorageWriter, utils::vixerr::Error};

pub struct AppendOnlyFileStorageWriter {
    file: File,
    last_offset: Mutex<u64>,
}

impl AppendOnlyFileStorageWriter {
    pub fn new(file: File, last_offset: u64) -> Arc<dyn AppendOnlyStorageWriter> {
        Arc::new(Self {
            file,
            last_offset: Mutex::new(last_offset),
        })
    }
}

impl AppendOnlyStorageWriter for AppendOnlyFileStorageWriter {
    fn write(&self, data: &[u8]) -> Result<(), Error> {
        let mut last_offset = self.last_offset.lock().unwrap();

        if let Err(e) = self.file.write_all_at(data, *last_offset) {
            return Error::code(FATAL_ERROR)
                .message("failed to append to file")
                .wrap(e)
                .throw();
        }

        *last_offset += data.len() as u64;
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
