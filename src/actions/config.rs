use crate::utils::{config::Config, config_utils, error::SnapError};

pub fn generate_config(local: bool) -> Result<Config, SnapError> {

    if local {
        return config_utils::build_local_config();
    }
    
    config_utils::build_global_config()
}