use std::{ffi::OsString, fs::{self, DirEntry, File, OpenOptions}, io::ErrorKind, path::{Path, PathBuf}, sync::Arc};

use dashmap::{DashMap, Entry};
use uuid::Uuid;

use crate::{errcode::FATAL_ERROR, storage::{Storage, file_storage_reader::FileStorageReader, file_storage_writer_append_only::AppendOnlyFileStorageWriter, file_storage_writer_write_once::WriteOnceFileStorageWriter}, utils::vixerr::Error};

pub struct FileStorage {
    file_reader_by_path: DashMap<PathBuf, Arc<File>>,
    base_dir: String,
    base_temp_dir: String,
}

impl FileStorage {
    pub fn new(base_dir: &str, base_temp_dir: &str) -> Arc<dyn Storage> {
        Arc::new(FileStorage {
            file_reader_by_path: DashMap::new(),
            base_dir: base_dir.to_string(),
            base_temp_dir: base_temp_dir.to_string(),
        })
    }
}

impl Storage for FileStorage {
    fn get_all_name_sorted(&self, store: &str) -> Result<Vec<String>, Error> {
        let path = Path::new(&self.base_dir)
            .join(store);

        let handle = match fs::read_dir(path) {
            Ok(e) => e,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
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

    fn get_append_only_writer(&self, store: &str, name: &str, last_offset: u64) -> Result<Box<dyn super::AppendOnlyStorageWriter>, Error> {
        let path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Error::code(FATAL_ERROR)
                    .message(format!("failed to create directory {}", parent.display()))
                    .wrap(e)
                    .throw();
            }
        }

        let file_result = OpenOptions::new()
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

        if let Err(e) = file.sync_all() {
            return Error::code(FATAL_ERROR)
                .message("failed to flush file")
                .wrap(e)
                .throw();
        }

        if let Some(parent) = path.parent() {
            let dir = match File::open(parent) {
                Ok(d) => d,
                Err(e) => return Error::code(FATAL_ERROR)
                    .message(format!("failed to open directory {}", parent.display()))
                    .wrap(e)
                    .throw(),
            };
            
            if let Err(e) = dir.sync_all() {
                return Error::code(FATAL_ERROR)
                    .message("failed to flush directory")
                    .wrap(e)
                    .throw();
            }
        }

        Ok(AppendOnlyFileStorageWriter::new(file, last_offset))
    }
    
    fn get_write_once_writer(&self, store: &str, name: &str) -> Result<Box<dyn super::WriteOnceStorageWriter>, Error> {
        let temp_path = Path::new(&self.base_temp_dir)
            .join(Uuid::new_v4().to_string());

        if let Some(temp_parent) = temp_path.parent() {
            if let Err(e) = fs::create_dir_all(temp_parent) {
                return Error::code(FATAL_ERROR)
                    .message(format!("failed to create temp directory {}", temp_parent.display()))
                    .wrap(e)
                    .throw();
            }
        }

        let commit_path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        if let Some(commit_parent) = commit_path.parent() {
            if let Err(e) = fs::create_dir_all(commit_parent) {
                return Error::code(FATAL_ERROR)
                    .message(format!("failed to create commit directory {}", commit_parent.display()))
                    .wrap(e)
                    .throw();
            }
        }

        let file_result = OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(&temp_path);

        let file = match file_result {
            Ok(f) => f,
            Err(e) => return Error::code(FATAL_ERROR)
                .message(format!("failed to open file {}", temp_path.display()))
                .wrap(e)
                .throw(),
        };

        Ok(WriteOnceFileStorageWriter::new(commit_path, temp_path, file))
    }
    
    fn get_reader(&self, store: &str, name: &str) -> Result<Box<dyn super::StorageReader>, Error> {
        let path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        let file_arc = match self.file_reader_by_path.entry(path.clone()) {
            Entry::Occupied(f) => Arc::clone(f.get()),
            Entry::Vacant(v) => {
                let file_result = OpenOptions::new()
                    .read(true)
                    .open(&path);

                let file_arc = match file_result {
                    Ok(f) => Arc::new(f),
                    Err(e) => return Error::code(FATAL_ERROR)
                        .message(format!("failed to open file {}", path.display()))
                        .wrap(e)
                        .throw(),
                };

                v.insert(file_arc.clone());
                file_arc
            },
        };

        Ok(FileStorageReader::new(file_arc))
    }

    fn close(&self, store: &str, name: &str) -> Result<(), Error> {
        let path = Path::new(&self.base_dir)
            .join(store)
            .join(name);

        self.file_reader_by_path.remove(&path);

        Ok(())
    }
}
