use std::{fs::File, os::unix::fs::FileExt, sync::Arc};

use crate::{errcode::FATAL_ERROR, storage::BlockWriter, utils::vixerr::Error};

pub struct FileWriter {
    name: String,
    file: Arc<File>,
}

pub fn new_file_writer(name: &str, file: Arc<File>) -> Box<dyn BlockWriter> {
    Box::new(FileWriter::new(name.to_string(), file))
}

impl BlockWriter for FileWriter {
    fn write(&self, offset: u64, data: &[u8]) -> Result<(), Error> {
        if let Err(e) = self.file.write_at(data, offset) {
            return Error::code(FATAL_ERROR)
                .message(format!("failed to write to file {}", &self.name))
                .wrap(e)
                .throw();
        };

        Ok(())
    }

    fn commit(self: Box<Self>) -> Result<(), Error> {
        if let Err(e) = self.file.sync_all() {
            return Error::code(FATAL_ERROR)
                .message(format!("failed to commit file {}", &self.name))
                .wrap(e)
                .throw();
        };

        Ok(())
    }
}

impl FileWriter {
    pub fn new(name: String, file: Arc<File>) -> Self {
        FileWriter {
            name: name.to_string(),
            file,
        }
    }
}
