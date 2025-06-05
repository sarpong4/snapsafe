
// I will use both timestamps and hashing here;
// for each file in the path, even if we have to into sub folders, we will hash the file content 
// and the last modified timestamp.
// if the timestamp and a file hasn't changed, skip it
// if we don't have a record of that file's timestamp, proceed to hashing and back it up, 
// if timestamp has changed, check for hash changes and either backup or skip

use std::{fs, io, path::Path};
use rpassword::prompt_password;

pub mod crypto;
pub mod gc;
pub mod snapshot;

fn read_password() -> String {
    if let Ok(pwd) = std::env::var("SNAPSAFE_PASSWORD") {
        return pwd;
    }

    prompt_password("Enter password: ").expect("Failed to read password")
}

fn get_nth_recent_json_snapshot(nth: usize, dir: &Path) -> io::Result<Option<String>> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().extension().map(|ext| ext == "json").unwrap_or(false)
            && e.metadata().map(|m| m.is_file()).unwrap_or(false)
        })
        .filter_map(|entry| {
            entry.metadata().ok().and_then(|meta| {
                meta.modified().ok().map(|modified| (modified, entry.path()))
            })
        }).collect();

    entries.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(entries.get(nth).map(|(_, path)| path.to_string_lossy().to_string()))
}

fn get_salt(dir: &Path) -> Vec<u8> {
    let salt_path = dir.join("key_salt");
    let salt = if salt_path.exists() {
        fs::read(&salt_path).expect("Cannot read salt file.")
    } else {
        let new_salt: [u8; 16] = rand::random();
        let _ = fs::write(&salt_path, &new_salt);
        return new_salt.to_vec()
    };

    salt
}

fn get_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound, 
        format!("No data backup available at specified origin path: 
        \nCheck that your path is correct and password is valid"))
}

pub fn backup_file(source: &str, target: &str) -> io::Result<()> {
    let password = read_password();

    let src = Path::new(source);
    let dest = Path::new(target);

    let blobs_dir = dest.join("blobs");
    let snapshot_dir = dest.join("snapshot");

    fs::create_dir_all(&blobs_dir)?;
    fs::create_dir_all(&snapshot_dir)?;

    // we need the latest json file if there is any
    let latest_json = if snapshot_dir.try_exists().unwrap() {
        get_nth_recent_json_snapshot(0, &snapshot_dir)?.map(|path| std::path::PathBuf::from(path))
    } else {
        None
    };

    let salt = get_salt(&dest);
    let key = crypto::derive_key(&password, &salt);
    let snap = snapshot::Snapshot::create(src, &blobs_dir, &key, latest_json.as_ref())?;
    let _ = snap.save(&blobs_dir, &snapshot_dir)?;

    println!("Backup completed successfully");

    Ok(())
}

pub fn restore(nth: u8, source: &str, target: &str) -> io::Result<()> {
    let password = read_password();

    let nth = (nth - 1) as usize;
    let src = Path::new(source);
    let output_dir = Path::new(target);

    if !output_dir.try_exists().unwrap_or(false) {
        let _ = fs::create_dir_all(&output_dir)?;
    }

    let salt = get_salt(&src);
    let key = crypto::derive_key(&password, &salt);

    let blobs_dir = src.join("blobs");
    let snapshot_dir = src.join("snapshot");
    
    let nth_snapshot = if snapshot_dir.try_exists().unwrap() {
        get_nth_recent_json_snapshot(nth, &snapshot_dir)?.map(|path| std::path::PathBuf::from(path))
    } else {
        None
    };

    if let Some(snapshot_path) = nth_snapshot {
        let snapshot_files = snapshot::Snapshot::from_json_to_snapshot(&snapshot_path)?.files;
        
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
                    }
                },
                Err(err) => {
                    let er = get_error();
                    eprintln!("[ERROR] Failed to decrypt file {:?} : {err}", path);
                    return Err(er); // error indistinguishability.
                }
            }
        }
    }
    else {
        let err = get_error();
        eprintln!("Failed to restore: \n{err}");
        return Err(err);

    }

    println!("Restore to {target} completed.");

    Ok(())
}

pub fn delete(nth: u8, origin: String) {
    todo!("Delete not implemented")
}

fn list_from_defined_path(path: &str) {
    todo!("list not implemented");
}

pub fn list(path: Option<String>) {
    match path {
        Some(file_path) => list_from_defined_path(&file_path),
        None => list_from_defined_path("")
    }
}
