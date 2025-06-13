use std::{fs, io, path::{Path, PathBuf}};

use crate::{actions::crypto, utils::{self, snapshot::Snapshot}};

pub fn restore(nth: usize, src: &Path, output_dir: &Path) -> io::Result<()> {
    let password = utils::read_password();

    if !output_dir.try_exists().unwrap_or(false) {
        let _ = fs::create_dir_all(&output_dir)?;
    }

    let salt = utils::get_salt(&src);
    let key = crypto::derive_key(&password, &salt);

    let blobs_dir = src.join("blobs");
    let snapshot_dir = src.join("snapshot");
    
    let nth_snapshot = if snapshot_dir.try_exists().unwrap() {
        utils::get_nth_recent_json_snapshot(nth, &snapshot_dir)?.map(|path| PathBuf::from(path))
    } else {
        None
    };

    if let Some(snapshot_path) = nth_snapshot {
        let snapshot = Snapshot::from_json_to_snapshot(&snapshot_path)?;
        let snapshot_files = snapshot.files;
        
        for (path, file_entry) in snapshot_files {
            let hash_path = blobs_dir.join(&file_entry.hash);

            let ciphertext = fs::read(&hash_path)?;
            let nonce_bytes = file_entry.nonce;

            match crypto::decrypt_file_bytes(&ciphertext, &key, &nonce_bytes) {
                Ok(decrytped) => {
                    let rel_target = output_dir.join(path);

                    if let Some(parent) = rel_target.parent() {
                        let _ = fs::create_dir_all(parent)?;
                    }
                    
                    if let Err(err) = fs::write(&rel_target, &decrytped) {
                        eprintln!("Could not write to restore file: {err}");
                        return Err(err);
                    }
                },
                Err(err) => {
                    let er = utils::get_error(utils::SnapError::Restore);
                    eprintln!("[ERROR] Failed to decrypt data at {:?} : {err}", path);
                    return Err(er); // error indistinguishability.
                }
            }
        }
        let mut registry = utils::get_registry();
        if let Some(entry) = utils::remove_snapshot(&mut registry, src.to_path_buf()) {
            let _ = registry.add_backup(entry);
            let _ = registry.save_to_file();
        }
        else {
            let err = utils::get_error(utils::SnapError::Delete);
            eprintln!("Snapshot with backup path: {:?} does not exist.", src);
            return Err(err);
        }

    }
    else {
        let err = utils::get_error(utils::SnapError::Restore);
        eprintln!("Failed to restore: \n{err}");
        return Err(err);

    }

    println!("Restore to {:?} completed.", output_dir.display());

    Ok(())
}
