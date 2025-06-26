
// I will use both timestamps and hashing here;
// for each file in the path, even if we have to into sub folders, we will hash the file content 
// and the last modified timestamp.
// if the timestamp and a file hasn't changed, skip it
// if we don't have a record of that file's timestamp, proceed to hashing and back it up, 
// if timestamp has changed, check for hash changes and either backup or skip

use std::{io, path::Path};

use crate::utils::{self, config::Config};

pub mod backup;
pub mod config;
pub mod delete;
pub mod restore;

pub fn backup(src: &Path, dest: &Path, comp: Option<String>, config: Option<Config>) -> io::Result<()> {
    backup::backup_data(src, dest, comp, config)
}

pub fn config(local: bool) -> io::Result<()> {
    config::generate_config(local)
}

pub fn restore(nth: u8, src: &Path, output_dir: &Path) -> io::Result<()> {
    let nth = (nth - 1) as usize;
    restore::restore(nth, src, output_dir)
}

pub fn delete(nth: u8, target: &Path, force: bool) -> io::Result<()> {
    // DO YOU REALLY WANT TO DELETE?
    let delete_confirm = if !force {
        let input = utils::prompt_for_input("Are you sure you want to permanently delete this backup? [y/N] ");
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

    let nth = (nth - 1) as usize;
    delete::delete_data(nth, target)    
}


pub fn list() -> io::Result<()> {
    let registry = utils::get_registry().registry;

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
