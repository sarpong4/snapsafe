// build global config
// build local config
// get configurations

use std::{fs::{self, File}, io::{self, stdin, stdout, Write}, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::utils::{get_error, get_registry_path, SnapError};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "general")]
    pub general: GeneralConfig,
    // pub security: SecurityConfig,
    // pub diff: DiffConfig,
    // pub cloud: CloudConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub registry_dir: String,
    pub compression: String,
    pub encryption: bool
}

// pub struct SecurityConfig {
//     encryption_algorithm: String,
//     key_derivation: String,
//     iterations: u128,
// }

// pub struct DiffConfig {
//     enabled: bool,
//     hash_algorithm: String,
// }

// pub struct CloudConfig {
//     enabled: bool,
//     provider: String,
//     bucket: String,
// }

impl Config {
    pub fn new() -> io::Result<Self> {
        let general = GeneralConfig::new();

        if general.is_none() {
            println!("An invalid registry path was provided. Please provide a correct one");
            println!("Restart the config process: snapsafe config");
            let err = get_error(SnapError::Config);
            return Err(err);
        }

        Ok(Self {
            general: general.unwrap()
        })
    }
}

impl GeneralConfig {
    pub fn new() -> Option<Self> {
        let registry = match get_registry_dir() {
            Some(dir) => dir,
            None => return None
        };

        let compression = match get_compression_type() {
            Some(comp) => comp,
            None => {
                println!("Compression Algorithm set to None. You can change it with snapsafe --config");
                "none".to_string()
            }
        };

        Some(Self {
            registry_dir: registry,
            compression,
            encryption: true
        })
    } 
}


fn get_compression_type() -> Option<String> {
    print!("Provide the compression algorithm you prefer [gzip, zlib, brotli, zstd, lzma]: ");
    stdout().flush().unwrap();

    let mut input = String::new();

    let compression = match stdin().read_line(&mut input) {
        Ok(_) => {
            let response = input.trim().to_lowercase();
            if response.is_empty() {
                return None
            }
            response
        }
        Err(_) => {
            return None
        }
    };

    Some(compression)
}

fn get_registry_dir() -> Option<String> {
    print!("Provide your registry directory path or press D for default: ");
    stdout().flush().unwrap();

    let mut input = String::new();

    let registry_dir = match stdin().read_line(&mut input) {
        Ok(_) => {
            let response = input.trim().to_lowercase();
            if response.is_empty() {
                return None
            }
            response
        }
        Err(_) => {
            return None
        }
    };

    if registry_dir == "d" {
        Some(get_registry_path())
    }
    else {
        Some(registry_dir)
    }
}

pub fn build_global_config() -> io::Result<Config> {
    let home_dir = dirs::home_dir().unwrap();
    let snapsafe_dir = home_dir.join(".snapsafe");

    if !snapsafe_dir.exists() {
        let _ = fs::create_dir_all(&snapsafe_dir);
    }

    let config_path = snapsafe_dir.join("snapsafe.toml");

    build_config(config_path)
}

pub fn build_local_config() -> io::Result<Config> {
    let home_dir = PathBuf::from("./");
    let config_path = home_dir.join("snapsafe.toml");

    build_config(config_path)
}

fn build_config(config_path: PathBuf) -> io::Result<Config> {
    let config = Config::new()?;
    
    let toml_string = toml::to_string_pretty(&config).expect("Serialization Failed");
    let mut file = File::create(&config_path)?;
    let _ = file.write_all(toml_string.as_bytes());
    // find a way to write to a .toml file

    Ok(config)
}

/// Check local directory for a config file.
/// If no file is found check global director for the file
/// if both have no config file return `None`
/// Else `Some(Config)`
pub fn get_config() -> Option<Config> {
    let local_path = PathBuf::from("./snapsafe.toml");

    if local_path.exists() {
        let content = fs::read_to_string(&local_path).unwrap();
        let config: Config = toml::from_str(&content)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err)).unwrap();

        return Some(config);
    }

    let home_dir = dirs::home_dir().unwrap();
    let global_path = home_dir.join(".snapsafe").join("snapsafe.toml");

    if global_path.exists() {
        let content = fs::read_to_string(&global_path).unwrap();
        let config: Config = toml::from_str(&content)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err)).unwrap();

        return Some(config);
    }

    None
}
