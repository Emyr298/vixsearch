use std::{fs::{File, OpenOptions}, os::unix::fs::FileExt, path::Path, sync::Arc};

use dashmap::DashMap;

use crate::{errcode::FATAL_ERROR, storage::{BlockWriter, errors::NAME_NOT_FOUND, file_writer::new_file_writer, storage::BlockStorage}, utils::vixerr::Error};

pub struct FileStorage {
    file_by_name: DashMap<String, Arc<File>>,
    base_dir: String,
}

pub fn new_file_storage(base_dir: String) -> Arc<dyn BlockStorage> {
    return Arc::new(FileStorage::new(base_dir));
}

impl BlockStorage for FileStorage {
    fn size(&self, name: &str) -> Result<u64, Error> {
        let Some(file) = self.file_by_name.get(name) else {
            return Error::code(FATAL_ERROR).message(NAME_NOT_FOUND).throw();
        };

        let file_metadata = match file.metadata() {
            Ok(val) => val,
            Err(err) => return Error::code(FATAL_ERROR)
                .message("failed to open file")
                .wrap(err)
                .throw(),
        };
        
        Ok(file_metadata.len())
    }

    fn read(&self, name: &str, offset: u64, size: u64) -> Result<Vec<u8>, Error> {
        let Some(file) = self.file_by_name.get(name) else {
            return Error::code(FATAL_ERROR).message(NAME_NOT_FOUND).throw();
        };

        let mut buf = vec![0u8; size as usize];
        if let Err(e) = file.read_exact_at(&mut buf, offset) {
            return Error::code(FATAL_ERROR)
                .message(format!("failed to read from file {}", name))
                .wrap(e)
                .throw();
        };

        Ok(buf)
    }

    fn writer(&self, name: &str) -> Result<Box<dyn BlockWriter>, Error> {
        let Some(file) = self.file_by_name.get(name) else {
            return Error::code(FATAL_ERROR)
                .message(NAME_NOT_FOUND)
                .throw();
        };

        let file_arc = Arc::clone(&file);
        Ok(new_file_writer(name, file_arc))
    }
}

impl FileStorage {
    fn new(base_dir: String) -> Self {
        return FileStorage {
            file_by_name: DashMap::new(),
            base_dir,
        };
    }

    pub fn open(&self, name: &str) -> Result<(), Error> {
        let path = Path::new(&self.base_dir).join(name);

        let file_result = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path);

        let file = match file_result {
            Ok(f) => f,
            Err(e) => return Error::code(FATAL_ERROR)
                .message(format!("failed to open file {}", path.display()))
                .wrap(e)
                .throw(),
        };

        self.file_by_name.insert(name.to_string(), Arc::new(file));
        Ok(())
    }

    pub fn close(&self, name: &str) -> Result<(), Error> {
        self.file_by_name.remove(name);
        Ok(())
    }
}
