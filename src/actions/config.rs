use std::io;

use crate::utils::config_utils;

pub fn generate_config(local: bool) -> io::Result<()> {

    if local {
        let _ = config_utils::build_local_config();
    } else {
        let _ = config_utils::build_global_config();
    };

    Ok(())
}