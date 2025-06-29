use std::{fs, io, path::{Path, PathBuf}};

use crate::{crypto, utils::{self, snapshot::Snapshot, SnapError}};

pub fn delete_data(nth: usize, target: &Path) -> io::Result<()> {
    let password = utils::read_password();

    let mut registry = utils::get_registry();
    let entry = registry.find_entry_from_dest(target.to_path_buf());

    let password_hash = utils::hash_password(&password);
    let comp_from_entry = utils::compare_password(entry, &password_hash, SnapError::Delete);

    if let Err(err) = comp_from_entry {
        return Err(err);
    }

    let salt = utils::get_salt(&target);
    let key = crypto::derive_key(&password, &salt);

    let blob_dir = target.join("blobs");
    let snapshot_dir = target.join("snapshot");

    if !blob_dir.exists() || !snapshot_dir.exists() {
        let err = utils::get_error(utils::SnapError::Delete);
        eprintln!("Target does not contain any backup");
        return Err(err);
    }

    let nth_snapshot = utils::get_nth_recent_json_snapshot(nth, &snapshot_dir)?.map(|p| PathBuf::from(p));

    if let Some(snap_path) = nth_snapshot {
        let snapshot = Snapshot::from_json_to_snapshot(&snap_path)?;
        let snapshot_files = snapshot.files;

        for (path, file) in snapshot_files {
            if file.isupdated {
                let hash_path = blob_dir.join(&file.hash);

                let ciphertext = fs::read(&hash_path)?;
                let nonce_bytes = file.nonce;

                match crypto::decrypt_file_bytes(&ciphertext, &key, &nonce_bytes) {
                    Ok(_) => {
                        if let Err(_) = fs::remove_file(&hash_path) {
                            let err = utils::get_error(utils::SnapError::Delete);
                            return Err(err);
                        }

                        if hash_path.exists() {
                            eprintln!("Hash file {:?} was not deleted properly.", hash_path);
                            let err = utils::get_error(utils::SnapError::Delete);
                            return Err(err);
                        }
                    },
                    Err(err) => {
                        eprintln!("Failed to decrypt data at {:?}: {err}", path);
                        let er = utils::get_error(utils::SnapError::Delete);
                        return Err(er);
                    }
                }
            }
        }
        if let Err(_) = fs::remove_file(snap_path) {
            let err = utils::get_error(utils::SnapError::Delete);
            return Err(err);
        }
        
        
        if let Some(entry) = utils::remove_snapshot(&mut registry, target.to_path_buf()) {
            let _ = registry.add_backup(entry);
            let _ = registry.save_to_file();
        }
        else {
            let err = utils::get_error(utils::SnapError::Delete);
            eprintln!("Snapshot with backup path: {:?} does not exist.", target);
            return Err(err);
        }
    }
    else {
        let err = utils::get_error(utils::SnapError::Delete);
        eprintln!("Failed to delete backup: Invalid password or unreadable metadata: {err}");
        return Err(err);
    }

    println!("Deletion complete.");

    Ok(())
}
