use std::{fs, io, path::{Path, PathBuf}};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::password::Password;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BackupEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub origin_path: PathBuf,
    pub backup_path: PathBuf,
    pub passsword: Password,
    pub snapshot_count: usize,
    pub compression_algorithm: String,
}

impl Default for BackupEntry {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().into(),
            timestamp: Utc::now(),
            origin_path: "source/some_file.txt".into(),
            backup_path: "target/some_file.bak".into(),
            passsword: Password::default(),
            snapshot_count: 1,
            compression_algorithm: "gzip".into()
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BackupRegistry {
    pub registry: Vec<BackupEntry>,
    pub registry_path: PathBuf
}

impl BackupEntry {
    pub fn new(timestamp: DateTime<Utc>, src: PathBuf, target: PathBuf, password: &Password, compression: String) -> Self {
        let id = Uuid::new_v4().to_string();

        Self { 
            id, 
            timestamp, 
            origin_path: src, 
            backup_path: target, 
            passsword: password.clone(), 
            snapshot_count: 1,
            compression_algorithm: compression,
        }
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
        let snapsafe_path = home_dir.join(".snapsafe");

        if !snapsafe_path.exists() {
            let _ = fs::create_dir_all(&snapsafe_path);
        }

        let registry_json = snapsafe_path.join("backup_registry.json");
        if !registry_json.exists() {
            let _ = fs::File::create(&registry_json);
        }
        Self { registry: Vec::new(), registry_path: registry_json }
    }

    pub fn build_test_registy(path: String) -> Self {
        let temp_dir = Path::new(&path);

        if !temp_dir.exists() {
            let _ = fs::create_dir_all(&temp_dir);
        }
        let temp_json = temp_dir.join("snapsafe_test_registry.json");
        
        if !temp_json.exists() {
            let _ = fs::File::create(&temp_json);
        }

        Self { registry: Vec::new(), registry_path: temp_json }
    }

    pub fn find_entry(&self, src: PathBuf, dest: PathBuf) -> Option<&BackupEntry> {
        self.registry.iter().find(|en| {
            en.origin_path == src && en.backup_path == dest
        })
    }

    /// Given a destination path, `dest`, find the entry saved in the registry
    /// 
    /// Note: Destination path is always unique, when a backup is made to that path from source,
    /// the idea is that you cannot make another backup to that path.
    pub fn find_entry_from_dest(&self, dest: PathBuf) -> Option<&BackupEntry> {
        self.registry.iter().find(|en| {
            en.backup_path == dest
        })
    }

    pub fn load_from_file(path: &PathBuf) -> io::Result<Self> {

        if !path.exists() {
            fs::File::create(path)?;
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

    pub fn remove_backup(&mut self, entry: BackupEntry) {
        let ent = self.registry.iter().enumerate().find(|en| en.1.id == entry.id);
        if let Some((ix, _)) = ent {
            self.registry.remove(ix);
        }
    }

    pub fn add_backup(&mut self, entry: BackupEntry) {
        let ent = self.registry.iter().enumerate().find(|en| en.1.id == entry.id);
        if let Some((ix, _)) = ent {
            self.registry.remove(ix);
        }
        if entry.snapshot_count > 0 {
            self.registry.push(entry);
        }
    }
}
