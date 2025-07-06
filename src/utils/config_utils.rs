use core::convert::From;
use std::{fs::{self, File}, io::{self, stdin, stdout, Write}, path::PathBuf};

use crate::utils::{self, config::{Config, GeneralConfig}, error::SnapError};

pub fn get_compression_type() -> Option<String> {
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

pub fn get_gc_limit() -> Option<usize> {
     print!("Indicate the number of versions per file you want to save upon backup. Default is 3: ");
    stdout().flush().unwrap();

    let mut input = String::new();

    let limit = match stdin().read_line(&mut input) {
        Ok(_) => {
            let response = input.trim();
            if response.is_empty() {
                return None
            }
            match response.parse::<usize>() {
                Ok(val) => val,
                Err(_) => return None,
            }
        }
        Err(_) => {
            return None
        }
    };

    Some(limit)
}

pub fn get_registry_dir() -> Option<String> {
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
        Some(utils::get_registry_path())
    }
    else {
        Some(registry_dir)
    }
}

pub fn build_global_config() -> Result<Config, SnapError> {
    let home_dir = dirs::home_dir().unwrap();
    let snapsafe_dir = home_dir.join(".snapsafe");

    if !snapsafe_dir.exists() {
        let _ = fs::create_dir_all(&snapsafe_dir);
    }

    let config_path = snapsafe_dir.join("snapsafe.toml");

    build_config(config_path)
}

pub fn build_test_config(registry: String) -> io::Result<Config> {
    let home_dir = dirs::home_dir().unwrap();
    let snapsafe_dir = home_dir.join(".snapsafe");

    if !snapsafe_dir.exists() {
        let _ = fs::create_dir_all(&snapsafe_dir);
    }

    let test_dir = snapsafe_dir.join("test");

    if !test_dir.exists() {
        let _ = fs::create_dir_all(&test_dir);
    }

    let config_path = test_dir.join("snapsafe.toml");

    let comp = "gzip";
    let limit = 3;

    let general = GeneralConfig::from((registry, comp.to_string(), limit));
    let config = Config::from(general);

    let toml_string = toml::to_string_pretty(&config).expect("Serialization Failed");
    let mut file = File::create(&config_path)?;
    let _ = file.write_all(toml_string.as_bytes());

    Ok(config)
}

pub fn build_local_config() -> Result<Config, SnapError> {
    let home_dir = PathBuf::from("./");
    let config_path = home_dir.join("snapsafe.toml");

    build_config(config_path)
}

fn build_config(config_path: PathBuf) -> Result<Config, SnapError> {
    let config = Config::new()?;
    
    let toml_string = if let Ok(str) = toml::to_string_pretty(&config){
        str
    }else {
        let err = SnapError::Config("Serialization Failed".into());
        return Err(err);
    };

    let mut file = File::create(&config_path)?;
    file.write_all(toml_string.as_bytes())?;

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
