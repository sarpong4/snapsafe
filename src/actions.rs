
// I will use both timestamps and hashing here;
// for each file in the path, even if we have to into sub folders, we will hash the file content 
// and the last modified timestamp.
// if the timestamp and a file hasn't changed, skip it
// if we don't have a record of that file's timestamp, proceed to hashing and back it up, 
// if timestamp has changed, check for hash changes and either backup or skip

use std::{fs, io, path::Path, time::SystemTime};
use rpassword::prompt_password;

pub mod crytpo;
pub mod gc;
pub mod snapshot;

pub fn read_password() -> String {
    if let Ok(pwd) = std::env::var("SNAPSAFE_PASSWORD") {
        return pwd;
    }

    prompt_password("Enter password: ").expect("Failed to read password")
}

pub fn most_recent_json_snapshot(dir: &Path) -> io::Result<Option<String>> {
    let mut newest: Option<(SystemTime, String)> = None;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            if let Ok(modified) = metadata.modified() {
                let path_str = entry.path().to_string_lossy().to_string();
                match &newest {
                    Some((latest_time, _)) if *latest_time >= modified => {},
                    _ => newest = Some((modified, path_str)),
                };
            }
        }
    }

    Ok(newest.map(|(_, path)| path))
}

pub fn backup_file(source: &str, target: &str) -> io::Result<()> {
    let src = Path::new(source);
    let dest = Path::new(target);

    let blobs_dir = dest.join("blobs");
    let snapshot_dir = dest.join("snapshot");

    fs::create_dir_all(&blobs_dir)?;
    fs::create_dir_all(&snapshot_dir)?;

    // we need the latest json file if there is any
    let latest_json = if snapshot_dir.try_exists().unwrap() {
        most_recent_json_snapshot(&snapshot_dir)?.map(|path| std::path::PathBuf::from(path))
    } else {
        None
    };

    let password = read_password();
    let salt_path = dest.join("key_salt");
    let salt = if salt_path.exists() {
        fs::read(&salt_path)?
    } else {
        let new_salt: [u8; 16] = rand::random();
        fs::write(&salt_path, &new_salt)?;
        new_salt.to_vec()
    };

    let key = crytpo::derive_key(&password, &salt);
    let snap = snapshot::Snapshot::create(src, &blobs_dir, &key, latest_json.as_ref())?;
    // println!("Snapshot: {:?}", snap);
    let _ = snap.save(&blobs_dir, &snapshot_dir)?;

    println!("Backup completed successfully"); // curios why this part does not printout.🤔

    Ok(())
}

pub fn restore(source: &str, snapshot_id: u8, target: &str) {
    println!("About to restore: {source}");
    todo!("restore not implemented");

}

pub fn delete(snapshot_id: u8) {
    println!("About to delete snapshot with id: {snapshot_id}");
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

pub fn get_file_path_from_id(snapshot_id: u8) -> String {
    todo!("Not implemented");
}