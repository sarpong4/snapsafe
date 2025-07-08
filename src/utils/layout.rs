use std::{fs, path::{Path, PathBuf}};

use crate::utils::error::SnapError;

pub struct SnapshotLayout {
    pub base: PathBuf,
    pub blobs: PathBuf,
    pub snapshots: PathBuf,
}

impl SnapshotLayout {
    pub fn initialize(base: impl AsRef<Path>) -> Result<Self, SnapError> {
        let base = base.as_ref().to_path_buf();
        let blobs = base.join("blobs");
        let snapshots = base.join("snapshots");

        fs::create_dir_all(&blobs)?;
        fs::create_dir_all(&snapshots)?;

        Ok(Self { base, blobs, snapshots })
    }

    pub fn validate(&self) -> Result<(), SnapError> {
        if !self.blobs.is_dir() || !self.snapshots.is_dir() {
            return Err(SnapError::InvalidSnapshotLayout(self.base.clone()));
        }

        Ok(())
    }
}
