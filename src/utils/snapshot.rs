use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use std::{collections::HashMap, fs, io::{self, Write}, path::{Path, PathBuf}, time::SystemTime};

use crate::crypto;
use crate::utils::gc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Snapshot {
    pub timestamp: DateTime<Utc>, // update to timestamp type
    pub files: HashMap<PathBuf, FileEntry> // relative file_path -> filehash
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub hash: String,
    pub nonce: [u8; 12],
    pub modified: SystemTime,
    pub isupdated: bool
}

impl Snapshot {
    pub fn create(src: &Path, target: &Path, key: &[u8], latest_json_path: Option<&PathBuf>) -> io::Result<Self> {
        let mut files = HashMap::<PathBuf, FileEntry>::new();
        let mut old_files = HashMap::<PathBuf, FileEntry>::new();

        // get the latest json for this target path.
        let last_state = if let Some(json_path) = latest_json_path {
            Some(Snapshot::from_json_to_snapshot(json_path.as_path())?)
        } else {
            None
        };
        let last_state = last_state.as_ref();

        for entry in WalkDir::new(src).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            let metadata = entry.metadata()?;

            if path.is_file() {
                let rel_path = path.strip_prefix(src).unwrap().to_path_buf();
                let content = fs::read(path)?;
                let hash = Sha256::digest(&content);
                
                let prev_state = match last_state {
                    Some(snap) => snap.files.get(&rel_path),
                    None => None
                };


                match prev_state {
                    Some(f) if metadata.modified()? == f.modified => {
                        let mut file = f.clone();
                        file.isupdated = false;
                        old_files.insert(rel_path, file);
                    }
                    Some(file_entry) if file_entry.hash == format!("{:x}", &hash) => {
                        let mut file = file_entry.clone();
                        file.isupdated = false;
                        old_files.insert(rel_path, file);
                    },
                    _ => {
                        let (ciphertext, nonce) = crypto::encrypt_file_bytes(&content, key);

                        let hash_hex = format!("{:x}", hash);
                        let blob_path = target.join(&hash_hex);

                        fs::write(&blob_path, ciphertext)?;

                        files.insert(rel_path, FileEntry { hash: hash_hex, nonce, modified: SystemTime::now(), isupdated: true });
                    }
                }
            }
        }

        if files.is_empty() {
            let err = io::Error::new(
                io::ErrorKind::InvalidData, 
                format!("No File changes and hence backup aborted."));
            return Err(err);
        }
        else {
            files.extend(old_files);
        }
        
        Ok(
            Self{ timestamp: Utc::now(), files }
        )
    }

    pub fn save(&self, blobs_dir: &PathBuf, snapshot_dir: &Path) -> io::Result<()> {
        let safe_timestamp = self.timestamp.format("%Y-%m-%dT%H-%M-%S").to_string();
        let file_path = snapshot_dir.join(format!("{safe_timestamp}.json"));

        let mut gc = gc::GarbageCollector::new(blobs_dir.clone(), 3);
        
        if !&self.files.is_empty() {
            for (file_path, file_entry) in &self.files {
                gc.register_file(&file_path, &file_entry.hash)?;
            }

            let mut file = fs::File::create(&file_path).expect("Failed to create snapshot file");
            let json = serde_json::to_string_pretty(&self)?;
            file.write_all(json.as_bytes()).unwrap_or_else(|err| {
                eprintln!("Could not write snapshot file: {err}");
            });
        }
        else {
            println!("Nothing to add to json, state did not change for any file");
        }

        Ok(())
    }

    pub fn from_json_to_snapshot(json_path: &Path) -> io::Result<Self> {
        let content = fs::read(json_path).map_err(|err| {
            eprintln!("Could not content of {:?}: {err}", json_path);
            io::Error::new(io::ErrorKind::InvalidInput, err)
        })?;

        let data = serde_json::from_slice::<Snapshot>(&content)
            .map_err(|err| {
                eprintln!("Could not deserialize json: {err}");
                io::Error::new(io::ErrorKind::InvalidData, err)
            })?;

        Ok(data)
    }
}


