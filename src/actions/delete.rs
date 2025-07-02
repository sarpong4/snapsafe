use std::{fs, path::{Path, PathBuf}};

use crate::{crypto, utils::{self, error::SnapError, snapshot::Snapshot}};

pub fn delete_data(nth: usize, target: &Path) -> Result<(), SnapError> {
    let password = utils::read_password()?;

    let mut registry = utils::get_registry();
    let entry = registry.find_entry_from_dest(target.to_path_buf());

    if let Some(ent) = entry {
        let backup_password = &ent.password;
        let verify = backup_password.verify(&password);
        if let Err(err) = verify {
            return Err(SnapError::Password(err));
        }else if let Ok(false) = verify {
            return Err(SnapError::Password(crypto::password::PasswordError::IncorrectPassword));
        }
    }else {
        return Err(SnapError::Delete("Target provided does not exist.".into()));
    };

    let salt = utils::get_salt(&target);
    let key = crypto::derive_key(&password, &salt);

    let blob_dir = target.join("blobs");
    let snapshot_dir = target.join("snapshot");

    if !blob_dir.exists() || !snapshot_dir.exists() {
        return Err(SnapError::Delete("Target does not contain any backup".into()));
    }

    let nth_snapshot = utils::get_nth_recent_json_snapshot(nth, &snapshot_dir)?.map(|p| PathBuf::from(p));

    if let Some(snap_path) = nth_snapshot {
        let snapshot = Snapshot::from_json_to_snapshot(&snap_path)?;
        let snapshot_files = snapshot.files;

        for (_, file) in snapshot_files {
            if file.isupdated {
                let hash_path = blob_dir.join(&file.hash);

                let ciphertext = fs::read(&hash_path)?;
                let nonce_bytes = file.nonce;

                match crypto::decrypt_file_bytes(&ciphertext, &key, &nonce_bytes) {
                    Ok(_) => {
                        fs::remove_file(&hash_path)?;
                    },
                    Err(err) => {
                        let message = "Could not decrypt file";
                        return Err(SnapError::EncryptError(message.into(), err));
                    }
                }
            }
        }
        fs::remove_file(snap_path)?;

        let mut ent = entry.unwrap().clone();
        ent.remove_snapshot();
        registry.add_backup(ent);
        registry.save_to_file()?;
    }
    else {
        return Err(SnapError::Delete("Failed to delete backup: Invalid password or unreadble metadata.".into()));
    }

    println!("Deletion complete.");

    Ok(())
}
