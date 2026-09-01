use std::{fs::{self, File}, io::Write, path::PathBuf};

use crate::{errcode::FATAL_ERROR, storage::WriteOnceStorageWriter, utils::vixerr::Error};

pub struct WriteOnceFileStorageWriter {
    commit_path: PathBuf,
    temp_path: PathBuf,
    file: File,
}

impl WriteOnceFileStorageWriter {
    pub fn new(commit_path: PathBuf, temp_path: PathBuf, file: File) -> Box<dyn WriteOnceStorageWriter> {
        Box::new(Self {
            commit_path,
            temp_path,
            file,
        })
    }
}

impl WriteOnceStorageWriter for WriteOnceFileStorageWriter {
    fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        if let Err(e) = self.file.write_all(data) {
            return Error::code(FATAL_ERROR)
                .message("failed to write to file")
                .wrap(e)
                .throw();
        };

        Ok(())
    }

    fn commit(self: Box<Self>) -> Result<(), Error> {
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

        if let Some(parent_path) = self.commit_path.parent() {
            let parent = match File::open(parent_path) {
                Ok(f) => f,
                Err(e) => return Error::code(FATAL_ERROR)
                    .message(format!("failed to open directory {}", parent_path.display()))
                    .wrap(e)
                    .throw(),
            };

            if let Err(e) = parent.sync_all() {
                return Error::code(FATAL_ERROR)
                    .message("failed to flush directory")
                    .wrap(e)
                    .throw();
            }
        }

        Ok(())
    }
}

impl Drop for WriteOnceFileStorageWriter {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.temp_path);
    }
}
