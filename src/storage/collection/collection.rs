use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::storage::collection::entity::{Collection, parse_and_verify_content};
use crate::utils::vixerr::Error;
use crate::{collection, errcode};

struct StorageImpl {
    data_dir: String,
    metadata_filename: String,
}

pub fn new_storage(data_dir: &str, metadata_filename: &str) -> Box<dyn collection::Storage> {
    return Box::new(StorageImpl::new(data_dir, metadata_filename));
}

impl StorageImpl {
    fn new(data_dir: &str, metadata_filename: &str) -> Self {
        return StorageImpl {
            data_dir: data_dir.to_string(),
            metadata_filename: metadata_filename.to_string(),
        };
    }
}

impl collection::Storage for StorageImpl {
    // TODO: ensure atomic create and delete
    fn create(&self, param: collection::CreateStorageParam) -> Result<(), Error> {
        let collection = Collection::from(param);

        let collection_json = serde_json::to_string(&collection)
            .map_err(|_| Error::new(errcode::SYSTEM_ERROR, "Failed to serialize collection"))?;

        let hash = crc32fast::hash(&collection_json.as_bytes());
        let file_content = format!("{:08x}:{}", hash, collection_json);

        let dir_path = PathBuf::from(self.data_dir.as_str()).join(collection.id);
        let path = dir_path.join(self.metadata_filename.as_str());

        fs::create_dir_all(&dir_path)
            .expect("PANIC: Failed to create collection directory on disk");

        let mut file = File::create(path).expect("PANIC: Failed to create collection file on disk");
        file.write_all(file_content.as_bytes())
            .expect("PANIC: Failed to write to collection file on disk");
        file.sync_all()
            .expect("PANIC: Failed to sync collection file on disk");

        Ok(())
    }

    fn delete(&self, id: &str) -> Result<(), Error> {
        let path = PathBuf::from(self.data_dir.as_str()).join(id);
        std::fs::remove_dir_all(path).expect("PANIC: Failed to remove collection file on disk");
        Ok(())
    }
    
    fn load(&self) -> Result<Vec<collection::Collection>, Error> {
        let root_path = Path::new(self.data_dir.as_str());
        if !root_path.exists() {
            println!("WARNING: Empty data directory");
            return Ok(vec![]);
        }

        let mut colls = Vec::<collection::Collection>::new();
        let entries = fs::read_dir(root_path).map_err(|_| Error::new(errcode::FATAL_ERROR, "failed to read_dir"))?;
        for entry in entries {
            let entry = entry.map_err(|_| Error::new(errcode::FATAL_ERROR, "failed to read entry"))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let metadata_path = path.join(&self.metadata_filename);
            if !metadata_path.exists() {
                return Err(Error::new(errcode::FATAL_ERROR, "missing metadata file"));
            }

            let content = fs::read_to_string(metadata_path).map_err(|_| Error::new(errcode::FATAL_ERROR, "failed to read metadata"))?;
            let json_str = parse_and_verify_content(&content)?;

            let coll = serde_json::from_str::<Collection>(json_str).map_err(|_| Error::new(errcode::FATAL_ERROR, "invalid metadata json"))?;
            colls.push(coll.try_into()?);
        }

        Ok(colls)
    }
}
