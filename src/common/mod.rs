use std::{fs, io::{self, Write}, path::{Path, PathBuf}};

use rpassword::prompt_password;

use crate::actions::registry::{BackupEntry, BackupRegistry};

pub enum SnapError {
    Backup,
    Restore,
    Delete,
    List,
}

pub fn clear_directory(path: &Path) -> io::Result<()> {
    if path.exists() && path.is_dir() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                fs::remove_dir_all(&entry_path)?;
            }
            else {
                fs::remove_file(&entry_path)?;
            }
        }
    }

    Ok(())
}

pub fn read_password() -> String {
    if let Ok(pwd) = std::env::var("SNAPSAFE_PASSWORD") {
        return pwd;
    }

    prompt_password("Enter password: ").expect("Failed to read password")
}

pub fn get_registry() -> BackupRegistry {
    
    let bkup_registry = 
        if let Ok(path) =  std::env::var("SNAPSAFE_TEST_REGISTRY") {
            BackupRegistry::build_test_registy(path)
        }
        else {
            BackupRegistry::new()
        };

    let backup_registry = BackupRegistry::load_from_file(&bkup_registry.registry_path)
                .unwrap_or(bkup_registry);

    backup_registry
}

pub fn hash_password(pw: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(pw.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn prompt_for_input(message: &str) -> Option<String> {
    print!("{message}");
    io::stdout().flush().unwrap();

    let mut input = String::new();

    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let response = input.trim().to_lowercase();
            if response.is_empty() {
                return None
            }
            Some(response)
        }
        Err(_) => None,
    }
}

pub fn get_nth_recent_json_snapshot(nth: usize, dir: &Path) -> io::Result<Option<String>> {
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

pub fn get_salt(dir: &Path) -> Vec<u8> {
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

pub fn get_error(err: SnapError) -> io::Error {
    match err {
        SnapError::Backup => {
            eprintln!("Backup Aborted!");
            io::Error::new(
                io::ErrorKind::NotFound, 
                format!("An Error occurred during Backup."))
        },
        SnapError::Restore | SnapError::Delete | SnapError::List => {
            eprintln!("Process Aborted!");
            io::Error::new(
                io::ErrorKind::NotFound, 
                "No data backup available at specified origin path: Check that your path is correct and password is valid".to_string())
        },
    }
}

pub fn remove_snapshot(registry: &BackupRegistry, dest: PathBuf) -> Option<BackupEntry> {
    let entry = registry.find_entry_from_dest(dest.to_path_buf());
    
    if let Some(en) = entry {
        let mut en = en.clone();
        en.remove_snapshot();
        return Some(en.clone());
    }

    None
}
