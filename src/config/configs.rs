// build global config
// build local config
// get configurations

use core::convert::From;
use std::io;

use serde::{Deserialize, Serialize};

use crate::{config, utils::{self, SnapError}};

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

impl From<GeneralConfig> for Config {
    fn from(value: GeneralConfig) -> Self {
        Self {
            general: value
        }
    }
}

impl From<(String, String)> for GeneralConfig {
    fn from(value: (String, String)) -> Self {
        let registry = value.0;
        let comp = value.1;

        Self {
            registry_dir: registry,
            compression: comp,
            encryption: true,
        }
    }
}

impl Config {
    pub fn new() -> io::Result<Self> {
        let general = GeneralConfig::new();

        if general.is_none() {
            println!("An invalid registry path was provided. Please provide a correct one");
            println!("Restart the config process: snapsafe config");
            let err = utils::get_error(SnapError::Config);
            return Err(err);
        }

        Ok(Self {
            general: general.unwrap()
        })
    }
}

impl GeneralConfig {
    pub fn new() -> Option<Self> {
        let registry = match config::get_registry_dir() {
            Some(dir) => dir,
            None => return None
        };

        let compression = match config::get_compression_type() {
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