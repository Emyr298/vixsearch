use std::{ffi::OsString, fs::{self, DirEntry, File, OpenOptions}, path::{Path, PathBuf}, sync::Arc};

use dashmap::{DashMap, Entry};
use uuid::Uuid;

use crate::{errcode::FATAL_ERROR, storage::{WormStorage, worm_file_reader::WormFileReader, worm_file_writer::WormFileWriter}, utils::vixerr::Error};

pub struct WormFileStorage {
    file_reader_by_path: DashMap<PathBuf, Arc<File>>,
    base_dir: String,
    base_temp_dir: String,
}

impl WormFileStorage {
    pub fn new(base_dir: &str, base_temp_dir: &str) -> Arc<dyn WormStorage> {
        Arc::new(WormFileStorage {
            file_reader_by_path: DashMap::new(),
            base_dir: base_dir.to_string(),
            base_temp_dir: base_temp_dir.to_string(),
        })
    }
}

impl WormStorage for WormFileStorage {
    fn get_all_commited_name_sorted(&self, store: &str) -> Result<Vec<String>, Error> {
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
            .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
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
    
    fn get_writer(&self, store: &str, name: &str) -> Result<Box<dyn super::WormWriter>, Error> {
        let temp_path = Path::new(&self.base_temp_dir)
            .join(Uuid::new_v4().to_string());

        let commit_path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        let file_result = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&temp_path);

        let file = match file_result {
            Ok(f) => f,
            Err(e) => return Error::code(FATAL_ERROR)
                .message(format!("failed to open file {}", temp_path.display()))
                .wrap(e)
                .throw(),
        };

        Ok(WormFileWriter::new(commit_path, temp_path, file))
    }
    
    fn get_reader(&self, store: &str, name: &str) -> Result<Box<dyn super::WormReader>, Error> {
        let path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        let file_arc = match self.file_reader_by_path.entry(path.clone()) {
            Entry::Occupied(f) => Arc::clone(f.get()),
            Entry::Vacant(_) => {
                let file_result = OpenOptions::new()
                    .read(true)
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

        Ok(WormFileReader::new(file_arc))
    }

    fn close(&self, store: &str, name: &str) -> Result<(), Error> {
        let path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        self.file_reader_by_path.remove(&path);

        Ok(())
    }
}
