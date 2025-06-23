use std::{fs, io, path::{Path, PathBuf}};

use crate::{actions::crypto, utils::{self, snapshot::Snapshot, SnapError}};

/// Restore the nth version of a backup at the location: `src`
/// Returns: `Ok(())` when successful or `Err` on any kind of failure.
/// 
/// if `nth` is one, we restore the latest backup. 
/// With default `GarbageCollectorConfig`, we keep 3 versions of each backup and hence 
/// nth is in the range 0..3
/// 
/// When a user defines a garbage collector config limit Lim, we define nth to be in the range `0..Lim`.
/// We make decompresssion based on the compression algorithm used during backup.
/// 
/// Decryption first occurs then decompression will take place.
/// the decompressed content is written to a file and saved in a path format similar to when backup occured. 
/// The `output_dir` is where the final files will be written to.
pub fn restore(nth: usize, src: &Path, output_dir: &Path) -> io::Result<()> {
    let password = utils::read_password();

    let mut registry = utils::get_registry();
    let entry = registry.find_entry_from_dest(src.to_path_buf());

    let comp = {
        if let Some(ent) = entry {
            ent.clone().compression_algorithm
        }
        else {
            let err = utils::get_error(SnapError::Restore);
            return Err(err);
        }
    };

    let engine = utils::generate_decompression_engine(comp);


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
