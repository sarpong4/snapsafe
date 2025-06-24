use std::{fs, io::{self, Write}, path::{Path, PathBuf}};

use rpassword::prompt_password;

use crate::{compress::{self, decompressor::DecompressionEngine, CompressionEngine}, config::{self, configs::Config}, utils::registry::{BackupEntry, BackupRegistry}};

pub mod gc;
pub mod registry;
pub mod snapshot;

pub enum SnapError {
    Command,
    Config,
    Backup,
    Restore,
    Delete,
    List,
}

/// Generate a compression engine from the algorithm information provided.
/// 
/// If `algorithm` is `None`, it means the user didn't provide an algorithm option
///  in the command line and there is no entry for this in the registry. Therefore we will use default algorithm `gzip`.
/// 
/// If we still find nothing (this is the first backup for this path) we then look 
/// through the local config folder for this definition and if it is still None
/// then we look at the global config folder
pub fn generate_compression_engine(algorithm: Option<String>) -> (CompressionEngine, String) {
    let comp = if let None = algorithm {
        "none".to_string()
    }else {
        algorithm.unwrap()
    };

    let engine = compress::build_compress_engine(comp.clone()).unwrap();
    (engine, comp)
}


/// Given the `algorithm` provided, build a decompression engine
/// We will get to know the kind of algorithm the compression used.
/// open to further refinement
pub fn generate_decompression_engine(algorithm: String) -> DecompressionEngine {

    compress::build_decompression_engine(algorithm).unwrap()
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

pub fn compare_password(entry: Option<&BackupEntry>, password_hash: &str, error: SnapError) -> io::Result<Option<String>> {
    // strict no password change enforcement.
    if let Some(ent) = entry {
        if password_hash != ent.passsword_hash {
            eprintln!("Destination password is different from the password you provided.");
            let err = get_error(error);
            return Err(err);
        }

        // use the compression algorithm on the BackupEntry even if it is different from the one 
        // like single-enforcement of password, you cannot change the compression algorithm once
        // it has been set.
        // the user provided now. We use the same algorithm throughout when we compress a 
        // given source -> destination
        return Ok(Some(ent.compression_algorithm.clone()));
    }

    Ok(None)
}

pub fn get_config() -> Config {
    let config = 
        if let Ok(registry) = std::env::var("TEST_CONFIG") {
            config::build_test_config(registry).unwrap()
        }
        else {
            config::get_config().unwrap()
        };

    config
}

pub fn get_registry_path() -> String {
    get_registry().registry_path.to_string_lossy().to_string()
}

pub fn generate_registry(path: String) -> BackupRegistry {
    BackupRegistry::load_from_file(&PathBuf::from(path)).unwrap()
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
        SnapError::Command => {
            eprintln!("Process Failed!");
            io::Error::new(
                io::ErrorKind::InvalidData,
                "An error occured before the command could execute"
            )
        },
        SnapError::Config => {
            eprintln!("Config Build Aborted");
            io::Error::new(
                io::ErrorKind::InvalidData,
                "An error occured before the config process could complete"
            )
        },
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

/// Remove a snapshot from the entry with the destination path (destination path is unique) 
/// from the registry.
/// 
/// 
pub fn remove_snapshot(registry: &BackupRegistry, dest: PathBuf) -> Option<BackupEntry> {
    let entry = registry.find_entry_from_dest(dest.to_path_buf());
    
    if let Some(en) = entry {
        let mut en = en.clone();
        en.remove_snapshot();
        return Some(en.clone());
    }

    None
}
