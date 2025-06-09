use std::{fs, io, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BackupEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub origin_path: PathBuf,
    pub backup_path: PathBuf,
    pub snapshot_count: usize,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BackupRegistry {
    pub registry: Vec<BackupEntry>,
    pub registry_path: PathBuf
}

impl BackupEntry {
    pub fn new(timestamp: DateTime<Utc>, src: PathBuf, target: PathBuf) -> Self {
        let id = Uuid::new_v4().to_string();

        Self { id, timestamp, origin_path: src, backup_path: target, snapshot_count: 1 }
    }

    pub fn add_snapshot(&mut self) {
        self.snapshot_count += 1;
    }

    pub fn remove_snapshot(&mut self) {
        self.snapshot_count -= 1;
    }
}

impl BackupRegistry {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let registry_path = home_dir.join(".snapsafe").join("backup_registry.json");

        let _ = fs::create_dir_all(registry_path.parent().unwrap());

        Self { registry: Vec::new(), registry_path }
    }

    pub fn find_entry(&self, src: PathBuf, dest: PathBuf) -> Option<&BackupEntry> {
        self.registry.iter().find(|en| {
            en.origin_path == src && en.backup_path == dest
        })
    }

    pub fn find_entry_from_dest(&self, dest: PathBuf) -> Option<&BackupEntry> {
        self.registry.iter().find(|en| {
            en.backup_path == dest
        })
    }

    pub fn load_from_file(path: &PathBuf) -> io::Result<Self> {

        if !path.exists() {
            return Ok(BackupRegistry::default());
        }

        let data = fs::read_to_string(path)?;
        let registry_data = serde_json::from_str(&data)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        Ok(registry_data)
    }

    pub fn save_to_file(&self) -> io::Result<()> {
        let data = serde_json::to_string_pretty(&self).expect("Could not prettify data");
        let _ = fs::write(&self.registry_path, data);

        Ok(())
    }

    pub fn add_backup(&mut self, entry: BackupEntry) {
        let ent = self.registry.iter().enumerate().find(|en| en.1.id == entry.id);
        if let Some((ix, _)) = ent {
            self.registry.remove(ix);
        }
        self.registry.push(entry);
    }
}