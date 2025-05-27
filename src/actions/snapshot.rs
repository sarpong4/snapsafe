use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use std::{collections::HashMap, fs, io, path::{Path, PathBuf}};

use crate::actions::crytpo;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub timestamp: DateTime<Utc>, // update to timestamp type
    pub files: HashMap<PathBuf, FileEntry> // relative file_path -> filehash
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub hash: String,
    pub nonce: [u8; 12]
}

impl Snapshot {
    pub fn create(src: &Path, target: &Path, key: &[u8]) -> io::Result<Self> {
        let mut files = HashMap::<PathBuf, FileEntry>::new();

        for entry in WalkDir::new(src).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                let rel_path = path.strip_prefix(src).unwrap().to_path_buf();
                let content = fs::read(path)?;
                let hash = Sha256::digest(&content);

                let (ciphertext, nonce) = crytpo::encrypt_file_bytes(&content, key);

                let hash_hex = format!("{:x}", hash);
                let blob_path = target.join(&hash_hex);
                let nonce_path = target.join(format!("{}.nonce", &hash_hex));

                fs::write(&blob_path, ciphertext)?;
                fs::write(&nonce_path, nonce)?;

                files.insert(rel_path, FileEntry { hash: hash_hex, nonce });
            }
        }

        Ok(
            Self{ timestamp: Utc::now(), files }
        )
    }

    pub fn save(&self, snapshots_dir: &Path) -> io::Result<()> {
        let file_path = snapshots_dir.join(format!("{}.json", self.timestamp.to_rfc3339()));
        let json = serde_json::to_string_pretty(&self)?;
        fs::write(file_path, json)?;        
        Ok(())
    }
}