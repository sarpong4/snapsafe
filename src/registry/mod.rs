use std::{collections::HashMap, fs, path::Path};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{crypto::password::Password, utils::error::SnapError};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BackupEntry {
    pub snapshot_id: String,
    pub origin: String,
    pub destination: String,
    pub timestamp: DateTime<Utc>,
    pub compression: String,
    pub encrypted: bool,
    pub size_bytes: u64,
    pub password: Password,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BackupRegistry {
    entries: HashMap<String, Vec<BackupEntry>>,
}

impl Default for BackupEntry {
    fn default() -> Self {
        Self {
            snapshot_id: uuid::Uuid::new_v4().into(),
            timestamp: Utc::now(),
            origin: "source/some_file.txt".into(),
            destination: "target/some_file.bak".into(),
            password: Password::default(),
            compression: "gzip".into(),
            encrypted: true,
            size_bytes: 123,
        }
    }
}

impl BackupRegistry {
    pub fn load(path: &Path) -> Result<Self, SnapError> {
        if !path.exists() {
            fs::File::create(path)?;
        }

        let data = fs::read_to_string(path)?;
        let registry_data = serde_json::from_str(&data)
            .map_err(|err| SnapError::SerializationError(format!("{:?}", err)))?;

        Ok(registry_data)
    }

    pub fn save(&self, path: &Path) -> Result<(), SnapError> {
        let data = serde_json::to_string_pretty(&self)
                .map_err(|err| SnapError::SerializationError(format!("{:?}", err)))?;
        let _ = fs::write(&path, data);

        Ok(())
    }

    pub fn add_entry(&mut self, entry: BackupEntry) -> Result<(), SnapError> {
        let dest = entry.destination.clone();
        let entries = self.entries.entry(dest.clone()).or_insert_with(Vec::new);
        entries.push(entry);

        Ok(())
    }

    pub fn get_latest_for(&self, dest: &Path) -> Result<Option<BackupEntry>, SnapError> {
        let dest = fs::canonicalize(dest)?.to_string_lossy().to_string();

        let entries = self.entries.get(&dest);

        let result = match entries {
            Some(entries) => {
                let entry = entries.iter().max_by(|a, b| a.timestamp.cmp(&b.timestamp)).cloned();
                entry
            },
            None => {
                None
            }
        };

        Ok(result)
    }

    pub fn get_nth_for(&self, dest: &Path, nth: usize) -> Result<Option<BackupEntry>, SnapError> {
        let dest = fs::canonicalize(dest)?.to_string_lossy().to_string();

        let entries = self.entries.get(&dest);

        let result = match entries {
                    Some(entries) => {
                        let mut sorted_entries = entries.clone();
                        sorted_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        let entry = sorted_entries.get(nth).cloned();
                        entry
                    },
                    None => {
                        None
                    }
                };

        Ok(result)
    }

    pub fn list_for(&self, dest: &Path) -> Result<Vec<BackupEntry>, SnapError> {
        let dest = fs::canonicalize(dest)?.to_string_lossy().to_string();

        let entries = self.entries.get(&dest).cloned().unwrap_or_else(Vec::new);

        Ok(entries)
    }
}
