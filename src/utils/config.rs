// build global config
// build local config
// get configurations

use core::convert::From;
use std::io;

use serde::{Deserialize, Serialize};

use crate::utils::{self, config_utils, SnapError};

#[derive(Serialize, Deserialize, Clone)]
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
    pub encryption: bool,
    pub gc_limit: usize
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

impl From<(String, String, usize)> for GeneralConfig {
    fn from(value: (String, String, usize)) -> Self {
        let registry = value.0;
        let comp = value.1;
        let limit = value.2;

        Self {
            registry_dir: registry,
            compression: comp,
            encryption: true,
            gc_limit: limit,
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

        println!("----------------------------");
        println!("BUILDING GENERAL BACKUP/RESTORE CONFIG");
        println!("-----------------------------\n");

        let registry =  match config_utils::get_registry_dir() {
            Some(dir) => dir,
            None => return None
        };


        let compression = match config_utils::get_compression_type() {
            Some(comp) => comp,
            None => {
                println!("Compression Algorithm set to None. You can change it with snapsafe config --global or snapsafe config --local");
                "none".to_string()
            }
        };

        let gc_limit = match config_utils::get_gc_limit() {
            Some(limit) => limit,
            None => {
                println!("Version Limit for each file is set to 3. You can change it with snapsafe config --global or snapsafe config --local");
                3
            }
        };

        Some(Self {
            registry_dir: registry,
            compression,
            encryption: true,
            gc_limit
        })
    }
}