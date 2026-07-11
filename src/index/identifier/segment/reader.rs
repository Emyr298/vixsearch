use std::{fs::{File, OpenOptions}, os::unix::fs::FileExt, path::Path};

use crate::{errcode, index::identifier::segment::constants::{BLOCK_HEADER_SIZE, FOOTER_MAGIC, FOOTER_SIZE, METADATA_MAGIC}, utils::vixerr::Error};



pub struct Reader {
    base_dir: String,
    segment_id: String,
    file: File,
}

impl Reader {
    pub fn new(base_dir: String, segment_id: String) -> Result<Self, Error> {
        let file_path = Path::new(&base_dir).join(format!("log_{}", &segment_id));
        let file_result = OpenOptions::new()
            .read(true)
            .open(file_path);

        let file = match file_result {
            Ok(val) => val,
            Err(err) => return Error::code(errcode::FATAL_ERROR)
                .message("failed to open file")
                .wrap(err)
                .throw(),
        };

        let metadata_offset = get_metadata_offset(&file)?;
        // let 

        Ok(Self {
            base_dir,
            segment_id,
            file,
        })
    }

}

fn get(file: &File, metadata_offset: u64, max_metadata_content_len: usize) -> Result<(), Error> {
    let mut metadata_header = [0u8; BLOCK_HEADER_SIZE];
    if let Err(err) = file.read_exact_at(&mut metadata_header, metadata_offset) {
        return Error::code(errcode::FATAL_ERROR)
            .message("failed to read metadata header")
            .wrap(err)
            .throw();
    };

    let magic = &metadata_header[0..4];
    if magic != METADATA_MAGIC {
        return Error::code(errcode::FATAL_ERROR)
            .message(format!("invalid file format: bad metadata magic"))
            .throw();
    }

    let expected_hash = u32::from_le_bytes(metadata_header[4..8].try_into().unwrap());
    let metadata_len_buf: [u8; 8] = metadata_header[8..FOOTER_SIZE].try_into().unwrap();
    let metadata_len: usize = u64::from_le_bytes(metadata_len_buf).try_into().unwrap();
    if metadata_len > max_metadata_content_len {
        return Error::code(errcode::FATAL_ERROR)
            .message(format!("invalid file format: bad metadata length"))
            .throw();
    }

    let mut metadata_content = vec![0u8; metadata_len];
    if let Err(err) = file.read_exact_at(&mut metadata_content, metadata_offset + (BLOCK_HEADER_SIZE as u64)) {
        return Error::code(errcode::FATAL_ERROR)
            .message("failed to read metadata content")
            .wrap(err)
            .throw();
    };

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(&metadata_len_buf);
    hasher.update(&metadata_content);
    let hash = hasher.finalize();

    if expected_hash != hash {
        return Error::code(errcode::FATAL_ERROR)
            .message(format!("invalid file format: metadata hash mismatch"))
            .throw();
    }

    let offsets_len: usize = u64::from_le_bytes(metadata_content[0..8].try_into().unwrap())
        .try_into()
        .unwrap();

    let offsets: Vec<u64> = rmp_serde::from_slice(&metadata_content[8..(8+offsets_len)]).unwrap();
    


    Ok(())
}

fn get_metadata_offset(file: &File) -> Result<u64, Error> {
    let file_metadata = match file.metadata() {
        Ok(val) => val,
        Err(err) => return Error::code(errcode::FATAL_ERROR)
            .message("failed to open file")
            .wrap(err)
            .throw(),
    };
    let file_size = file_metadata.len();
    let Some(footer_offset) = file_size.checked_sub(FOOTER_SIZE as u64) else {
        return Error::code(errcode::FATAL_ERROR)
            .message(format!("invalid file format: size is under footer size {}", FOOTER_SIZE))
            .throw()
    };

    let mut footer = [0u8; FOOTER_SIZE];
    if let Err(err) = file.read_exact_at(&mut footer, footer_offset) {
        return Error::code(errcode::FATAL_ERROR)
            .message("failed to read footer")
            .wrap(err)
            .throw();
    };

    let magic = &footer[0..4];
    if magic != FOOTER_MAGIC {
        return Error::code(errcode::FATAL_ERROR)
            .message(format!("invalid file format: bad footer magic"))
            .throw();
    }

    let expected_hash = u32::from_le_bytes(footer[4..8].try_into().unwrap());
    let metadata_offset: [u8; 8] = footer[8..FOOTER_SIZE].try_into().unwrap();
    let hash = crc32fast::hash(&metadata_offset);
    if expected_hash != hash {
        return Error::code(errcode::FATAL_ERROR)
            .message(format!("invalid file format: footer hash mismatch"))
            .throw();
    }

    Ok(u64::from_le_bytes(metadata_offset))
}
