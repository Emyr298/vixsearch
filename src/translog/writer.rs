use std::{
    fs::{File, OpenOptions},
    io::Error,
    path::PathBuf,
};

use crate::query::Action;
use crate::translog::constants;

pub trait TranslogWriter {
    fn append(&self, action: Action) -> Result<(), Error>;
    fn sync(&self, offset: i64) -> Result<(), Error>;
}

struct TranslogImpl {
    log: File,
}

pub fn new_translog(data_dir: String) -> impl TranslogWriter {
    let translog_path = PathBuf::from(data_dir.as_str()).join(constants::TRANSLOG_FILE_NAME);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(translog_path)
        .expect("Failed to open translog file");

    return TranslogImpl { log: file };
}

impl TranslogWriter for TranslogImpl {
    fn append(&self, _action: Action) -> Result<(), Error> {
        Ok(())
    }

    fn sync(&self, _offset: i64) -> Result<(), Error> {
        Ok(())
    }
}
