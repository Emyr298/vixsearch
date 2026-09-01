use std::{ffi::OsString, fs::{self, DirEntry, File, OpenOptions}, path::{Path, PathBuf}, sync::Arc};

use dashmap::{DashMap, Entry};

use crate::{errcode::FATAL_ERROR, storage::{LegacyStorageAccessor, LegacyStorage, legacy_file_storage_accessor::LegacyFileStorageAccessor}, utils::vixerr::Error};

pub struct LegacyFileStorage {
    file_by_path: DashMap<PathBuf, Arc<File>>,
    base_dir: String,
}

impl LegacyFileStorage {
    pub fn new(base_dir: &str) -> Arc<dyn LegacyStorage> {
        Arc::new(LegacyFileStorage {
            file_by_path: DashMap::new(),
            base_dir: base_dir.to_string(),
        })
    }
}

impl LegacyStorage for LegacyFileStorage {
    fn get_all_name_sorted(&self, store: &str) -> Result<Vec<String>, Error> {
        let path = Path::new(&self.base_dir)
            .join(store);

        let handle = match fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => return Error::code(FATAL_ERROR)
                .message("failed to read directory")
                .wrap(e)
                .throw()
        };

        let entries: Vec<DirEntry> = match handle.collect::<Result<Vec<_>, _>>() {
            Ok(e) => e,
            Err(e) => return Error::code(FATAL_ERROR)
                .message("failed to read directory")
                .wrap(e)
                .throw()
        };

        let result: Result<Vec<String>, OsString> = entries.into_iter()
            .map(|e| e.file_name().into_string())
            .collect();

        let mut names = match result {
            Ok(names) => names,
            Err(name_os) => {
                return Error::code(FATAL_ERROR)
                    .message(format!("invalid filename {:?}", name_os))
                    .throw()
            }
        };

        names.sort();
        Ok(names)
    }

    fn open(&self, store: &str, name: &str) -> Result<Box<dyn LegacyStorageAccessor>, Error> {
        let path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        let file_arc = match self.file_by_path.entry(path.clone()) {
            Entry::Occupied(f) => Arc::clone(f.get()),
            Entry::Vacant(_) => {
                let file_result = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path);

                match file_result {
                    Ok(f) => Arc::new(f),
                    Err(e) => return Error::code(FATAL_ERROR)
                        .message(format!("failed to open file {}", path.display()))
                        .wrap(e)
                        .throw(),
                }
            },
        };

        Ok(LegacyFileStorageAccessor::new(file_arc))
    }

    fn close(&self, store: &str, name: &str) -> Result<(), Error> {
        let path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        self.file_by_path.remove(&path);

        Ok(())
    }
}
