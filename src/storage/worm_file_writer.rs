use std::{fs::{self, File}, io::Write, path::PathBuf};

use crate::{errcode::FATAL_ERROR, storage::WormWriter, utils::vixerr::Error};

pub struct WormFileWriter {
    commit_path: PathBuf,
    temp_path: PathBuf,
    file: File,
}

impl WormFileWriter {
    pub fn new(commit_path: PathBuf, temp_path: PathBuf, file: File) -> Box<dyn WormWriter> {
        Box::new(Self {
            commit_path,
            temp_path,
            file,
        })
    }
}

impl WormWriter for WormFileWriter {
    fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        if let Err(e) = self.file.write_all(data) {
            return Error::code(FATAL_ERROR)
                .message("failed to write to file")
                .wrap(e)
                .throw();
        };

        Ok(())
    }

    fn commit(&self) -> Result<(), Error> {
        if let Err(e) = self.file.sync_all() {
            return Error::code(FATAL_ERROR)
                .message("failed to flush file")
                .wrap(e)
                .throw();
        };

        if let Err(e) = fs::rename(&self.temp_path, &self.commit_path) {
            return Error::code(FATAL_ERROR)
                .message("failed to atomic rename file")
                .wrap(e)
                .throw();
        }

        Ok(())
    }
}
