use std::{fs, path::{Path, PathBuf}};

use crate::{crypto, utils::{self, error::SnapError, snapshot::Snapshot}};

/// Restore the nth version of a backup at the location: `src`
/// Returns: `Ok(())` when successful or `Err(SnapError)` on any kind of failure.
/// 
/// if `nth` is 0, we restore the latest backup. 
/// With default `GarbageCollectorConfig`, we keep 3 versions of each backup and hence 
/// nth is in the range 0..3
/// 
/// When a user defines a garbage collector config limit, `Lim`, we define nth to be in the range `0..Lim`.
/// We make decompresssion based on the compression algorithm used during backup.
/// 
/// Decryption first occurs then decompression will take place.
/// the decompressed content is written to a file and saved in a path format similar to when backup occured. 
/// The `output_dir` is where the final files will be written to.
pub fn restore(nth: usize, src: &Path, output_dir: &Path) -> Result<(), SnapError> {
    let password = utils::read_password()?;

    let mut registry = utils::get_registry();
    let entry = registry.find_entry_from_dest(src.to_path_buf());

    let algorithm = if let Some(ent) = entry {
        let backup_password = &ent.password;
        let verify = backup_password.verify(&password);
        if let Err(err) = verify {
            return Err(SnapError::Password(err));
        }else if let Ok(false) = verify {
            return Err(SnapError::Password(crypto::password::PasswordError::IncorrectPassword));
        }
        ent.compression_algorithm.clone()
    }else {
        let message = "No backup available at path provided";
        return Err(SnapError::Restore(message.into()));
    };

    let engine = utils::generate_compression_engine(Some(algorithm))?.0;


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

                    let decompressed_content = engine.decompress(&decrytped)?;
                    
                    if let Err(err) = fs::write(&rel_target, &decompressed_content) {
                        return Err(SnapError::IOError(err));
                    }
                },
                Err(err) => {
                    let message = "Failed to decrypt target file";
                    return Err(SnapError::EncryptError(message.into(), err));
                }
            }
        }
        let mut ent = entry.unwrap().clone();
        ent.remove_snapshot();
        registry.add_backup(ent);
        registry.save_to_file()?;

        fs::remove_file(snapshot_path)?;

    }
    else {
        return Err(SnapError::Restore("Failed to restore".into()));
    }

    println!("Restore to {:?} completed.", output_dir.display());

    Ok(())
}
