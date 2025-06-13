use std::{collections::HashMap, fs, io, path::PathBuf};

pub struct GarbageCollector {
    version_index: HashMap<String, Vec<String>>,
    max_versions: usize,
    blobs_dir: PathBuf
}

impl GarbageCollector {
    pub fn new(blobs_dir: PathBuf, max_versions: usize) -> Self {
        Self {
            version_index: HashMap::new(),
            max_versions,
            blobs_dir
        }
    }

    pub fn register_file(&mut self, path: &PathBuf, hash: &str) -> io::Result<()> {
        let hashes = self.version_index.entry(path.to_string_lossy().to_string()).or_default();

        if hashes.first() != Some(&hash.to_string()) {
            hashes.insert(0, hash.to_string());
        }

        while hashes.len() > self.max_versions {
            if let Some(old_hash) = hashes.pop() {
                let blob_path = self.blobs_dir.join(&old_hash);
                let nonce_path = self.blobs_dir.join(format!("{}.nonce", &old_hash));

                if blob_path.exists() {
                    fs::remove_file(blob_path)?;
                }

                if nonce_path.exists() {
                    fs::remove_file(nonce_path)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_index(&self) -> &HashMap<String, Vec<String>> {
        &self.version_index
    }
}
