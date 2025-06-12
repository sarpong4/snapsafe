
// I will use both timestamps and hashing here;
// for each file in the path, even if we have to into sub folders, we will hash the file content 
// and the last modified timestamp.
// if the timestamp and a file hasn't changed, skip it
// if we don't have a record of that file's timestamp, proceed to hashing and back it up, 
// if timestamp has changed, check for hash changes and either backup or skip

use std::{fs, io, path::{Path, PathBuf}};

use crate::{actions::{registry::BackupEntry, snapshot::Snapshot}, common};

pub mod crypto;
pub mod gc;
pub mod registry;
pub mod snapshot;

pub fn backup_data(source: &str, target: &str) -> io::Result<()> {
    let password = common::read_password();

    let src = Path::new(source);
    let dest = Path::new(target);
    
    if !src.exists() {
        eprintln!("Source directory does not exist.");
        let err = common::get_error(common::SnapError::Backup);
        return Err(err);
    }

    if !dest.exists() {
        fs::create_dir_all(&dest)?;
    }

    let blobs_dir = dest.join("blobs");
    let snapshot_dir = dest.join("snapshot");

    fs::create_dir_all(&blobs_dir)?;
    fs::create_dir_all(&snapshot_dir)?;

    let mut registry = common::get_registry();
    let entry = registry.find_entry(src.to_path_buf(), dest.to_path_buf());

    let password_hash = common::hash_password(&password);

    // strict no password change enforcement.
    if let Some(ent) = entry {
        if password_hash != ent.passsword_hash {
            eprintln!("Backup destination already initialized with a different password. You cannot backup to this destination with a different password");
            let err = common::get_error(common::SnapError::Backup);
            return Err(err);
        }
    };

    // we need the latest json file if there is any
    let latest_json = if snapshot_dir.try_exists().unwrap() {
        common::get_nth_recent_json_snapshot(0, &snapshot_dir)?.map(|path| std::path::PathBuf::from(path))
    } else {
        None
    };

    let salt = common::get_salt(&dest);
    let key = crypto::derive_key(&password, &salt);
    let snap = snapshot::Snapshot::create(src, &blobs_dir, &key, latest_json.as_ref());
    
    if let Err(err) = snap {
        eprintln!("Backup Aborted!");
        return Err(err);
    }
    let snap = snap?;
    let _ = snap.save(&blobs_dir, &snapshot_dir)?;

    let entry = registry.find_entry(src.to_path_buf(), dest.to_path_buf());

    let ent;
    
    if let Some(en) = entry {
        let mut en = en.clone();
        en.add_snapshot();
        ent = en.clone();
    }
    else {
        ent = BackupEntry::new(snap.timestamp, src.to_path_buf(), dest.to_path_buf(), password_hash);
    }

    let _ = registry.add_backup(ent);
    let _ = registry.save_to_file();

    println!("Backup completed successfully");

    Ok(())
}


pub fn restore(nth: u8, source: &str, target: &str) -> io::Result<()> {
    let password = common::read_password();

    let nth = (nth - 1) as usize;
    let src = Path::new(source);
    let output_dir = Path::new(target);

    if !output_dir.try_exists().unwrap_or(false) {
        let _ = fs::create_dir_all(&output_dir)?;
    }

    let salt = common::get_salt(&src);
    let key = crypto::derive_key(&password, &salt);

    let blobs_dir = src.join("blobs");
    let snapshot_dir = src.join("snapshot");
    
    let nth_snapshot = if snapshot_dir.try_exists().unwrap() {
        common::get_nth_recent_json_snapshot(nth, &snapshot_dir)?.map(|path| PathBuf::from(path))
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
                    let er = common::get_error(common::SnapError::Restore);
                    eprintln!("[ERROR] Failed to decrypt data at {:?} : {err}", path);
                    return Err(er); // error indistinguishability.
                }
            }
        }

    }
    else {
        let err = common::get_error(common::SnapError::Restore);
        eprintln!("Failed to restore: \n{err}");
        return Err(err);

    }

    println!("Restore to {target} completed.");

    Ok(())
}

pub fn delete(nth: u8, origin: &str, force: bool) -> io::Result<()> {
    // DO YOU REALLY WANT TO DELETE?
    let delete_confirm = if !force {
        let input = common::prompt_for_input("Are you sure you want to permanently delete this backup? [y/N] ");
        match input {
            Some(response) => {
                response == "y" || response == "yes"
            }
            None => false,
        }
    } else {
        force
    };

    if !delete_confirm {
        println!("Delete Aborted!");
        return Ok(());
    }

    let password = common::read_password();

    let nth = (nth - 1) as usize;
    let target = Path::new(origin);

    if !target.try_exists().unwrap_or(false) {
        let err = common::get_error(common::SnapError::Delete);
        eprintln!("Target does not exist");
        return Err(err);
    }

    let salt = common::get_salt(&target);
    let key = crypto::derive_key(&password, &salt);

    let blob_dir = target.join("blobs");
    let snapshot_dir = target.join("snapshot");

    if !blob_dir.exists() || !snapshot_dir.exists() {
        let err = common::get_error(common::SnapError::Delete);
        eprintln!("Target does not contain any backup");
        return Err(err);
    }

    let nth_snapshot = if snapshot_dir.try_exists().unwrap() {
        common::get_nth_recent_json_snapshot(nth, &snapshot_dir)?.map(|p| PathBuf::from(p))
    } else {
        None
    };

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
                            let err = common::get_error(common::SnapError::Delete);
                            return Err(err);
                        }

                        if hash_path.exists() {
                            eprintln!("Hash file {:?} was not deleted properly.", hash_path);
                            let err = common::get_error(common::SnapError::Delete);
                            return Err(err);
                        }
                    },
                    Err(err) => {
                        eprintln!("Failed to decrypt data at {:?}: {err}", path);
                        let er = common::get_error(common::SnapError::Delete);
                        return Err(er);
                    }
                }
            }
        }
        if let Err(_) = fs::remove_file(snap_path) {
            let err = common::get_error(common::SnapError::Delete);
            return Err(err);
        }
        let mut registry = common::get_registry();
        println!("{:?}", registry);
        println!("Dest: {:?}", target);
        let entry = common::remove_snapshot(&mut registry, target.to_path_buf()).unwrap();

        let _ = registry.add_backup(entry);
        let _ = registry.save_to_file();
    }
    else {
        let err = common::get_error(common::SnapError::Delete);
        eprintln!("Failed to delete backup: Invalid password or unreadable metadata: {err}");
        return Err(err);
    }

    println!("Deletion complete.");

    Ok(())
}


pub fn list_from_registry() -> io::Result<()> {
    let registry = common::get_registry().registry;

    if registry.is_empty() {
        println!("No data has been backed up yet!");
    }
    else {
        println!("Listing All Backups 📦...");
        for entry in registry {
            println!(
                "- ID: {}\n Original Path: {:?}\n Backup Path: {:?}\n Created: {}\n Snapshots: {}",
                entry.id,
                entry.origin_path,
                entry.backup_path,
                entry.timestamp,
                entry.snapshot_count,
            )
        }
    }
    
    Ok(())
}
