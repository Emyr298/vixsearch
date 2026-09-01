use std::{fs::{self, File}, io::Write, os::unix::fs::FileExt, path::PathBuf};

use crate::{errcode::FATAL_ERROR, storage::AppendOnlyStorageWriter, utils::vixerr::Error};

pub struct AppendOnlyFileStorageWriter {
    file: File,
    last_offset: u64,
}

impl AppendOnlyFileStorageWriter {
    pub fn new(file: File, last_offset: u64) -> Box<dyn AppendOnlyStorageWriter> {
        Box::new(Self {
            file,
            last_offset,
        })
    }
}

impl AppendOnlyStorageWriter for AppendOnlyFileStorageWriter {
    fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        if let Err(e) = self.file.write_all_at(data, self.last_offset) {
            return Error::code(FATAL_ERROR)
                .message("failed to append to file")
                .wrap(e)
                .throw();
        }

        self.last_offset += data.len() as u64;

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
