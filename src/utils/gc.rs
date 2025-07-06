use std::{collections::HashMap, fs, io::{self, Write}, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::utils::{error::SnapError, snapshot::Snapshot};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GarbageCollector {
    version_index: HashMap<String, Vec<SnapshotReference>>,
    max_versions: usize,
    blobs_dir: PathBuf
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SnapshotReference {
    pub hash_file: String,
    snapshot_path: String,
}
#[derive(Serialize, Deserialize)]
pub struct GarbageLimit {
    pub gc: HashMap<String, GarbageCollector>,
}

impl From<(String, String)> for SnapshotReference {
    fn from(value: (String, String)) -> Self {
        let hash_file = value.0;
        let snapshot_path = value.1;

        Self {
            hash_file,
            snapshot_path
        }
    }
}

impl GarbageLimit {
    pub fn new() -> Self {
        let map = HashMap::<String, GarbageCollector>::new();

        Self {
            gc: map
        }
    }

    pub fn add_garbage_collector_to_limit(&mut self, gc: GarbageCollector) {
        let map = &mut self.gc;
        let path = gc.blobs_dir.to_string_lossy().to_string();
        map.insert(path, gc);
    }

    pub fn get_gc_from_limit(&self, path: String) -> Option<&GarbageCollector> {
        let map = &self.gc;

        map.get(&path)
    }

    pub fn save(&self) -> Result<(), SnapError> {
        
        if !&self.gc.is_empty() {
            let home_dir = dirs::home_dir().unwrap();
            let snapsafe_dir = home_dir.join(".snapsafe");

            if !snapsafe_dir.exists() {
                let _ = fs::create_dir_all(&snapsafe_dir);
            }

            let gc_path = snapsafe_dir.join("gc.json");
            let mut file = fs::File::create(&gc_path).expect("Failed to create gc file");
            let json = serde_json::to_string_pretty(&self).expect("Could not prettify data");
            file.write_all(json.as_bytes())?;
        }
        else {
            println!("Nothing to add to json, state did not change for any file");
        }

        Ok(())
    }

    pub fn from_json_to_gc() -> io::Result<Self> {
        let home_dir = dirs::home_dir().unwrap();
        let snapsafe_dir = home_dir.join(".snapsafe");
        let gc_path = snapsafe_dir.join("gc.json");

        let content = fs::read(&gc_path)?;

        let data = serde_json::from_slice::<GarbageLimit>(&content)?;

        Ok(data)
    }
}

impl GarbageCollector {
    pub fn new(blobs_dir: PathBuf, max_versions: usize) -> Self {
        Self {
            version_index: HashMap::new(),
            max_versions,
            blobs_dir
        }
    }

    pub fn register_file(&mut self, path: &PathBuf, hash: &str, snap_path: &PathBuf) -> io::Result<()> {
        let hashes = self.version_index.entry(path.to_string_lossy().to_string()).or_default();

        let first_hash = if let Some(s_reference) = hashes.first() {
            s_reference.hash_file.clone()
        }else {
            String::new()
        };

        if first_hash != hash.to_string() {
            let path = snap_path.to_string_lossy().to_string();
            let snap_ref = SnapshotReference::from((hash.to_string(), path));
            hashes.insert(0, snap_ref);
        }

        while hashes.len() > self.max_versions {
            if let Some(old_ref) = hashes.pop() {
                let hash_file = old_ref.hash_file;
                let snap_file = old_ref.snapshot_path;
                let blob_path = self.blobs_dir.join(&hash_file);

                if blob_path.exists() {
                    fs::remove_file(blob_path)?;
                }

                let snap_path = PathBuf::from(snap_file);

                let mut snapshot = Snapshot::from_json_to_snapshot(&snap_path)?;
                let mut files = snapshot.files;
                files.remove(&PathBuf::from(hash_file));

                snapshot.files = files;

                snapshot.save_snapshot(&snap_path)?;
            }
        }

        Ok(())
    }

    pub fn get_index(&self) -> &HashMap<String, Vec<SnapshotReference>> {
        &self.version_index
    }
}
