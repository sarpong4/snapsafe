
// I will use both timestamps and hashing here;
// for each file in the path, even if we have to into sub folders, we will hash the file content 
// and the last modified timestamp.
// if the timestamp and a file hasn't changed, skip it
// if we don't have a record of that file's timestamp, proceed to hashing and back it up, 
// if timestamp has changed, check for hash changes and either backup or skip

use std::{fs, io, path::Path};
use rpassword::prompt_password;

mod snapshot;
mod crytpo;


pub fn backup_file(source: &str, target: &str) -> io::Result<()> {
    let src = Path::new(source);
    let dest = Path::new(target);

    let blobs_dir = dest.join("blobs");
    let snapshot_dir = dest.join("snapshot");

    fs::create_dir_all(&blobs_dir)?;
    fs::create_dir_all(&snapshot_dir)?;

    let password = prompt_password("Enter password: ").unwrap();
    let salt_path = dest.join("key_salt");
    let salt = if salt_path.exists() {
        fs::read(&salt_path)?
    } else {
        let new_salt: [u8; 16] = rand::random();
        fs::write(&salt_path, &new_salt)?;
        new_salt.to_vec()
    };

    let key = crytpo::derive_key(&password, &salt);
    let snap = snapshot::Snapshot::create(src, &blobs_dir, &key)?;
    snap.save(&snapshot_dir)?;

    println!("Backup completed successfully");

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