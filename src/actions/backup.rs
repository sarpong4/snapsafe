use std::{fs, path::Path};

use crate::{crypto::{self, password::{Password, PasswordPolicy}}, utils::{self, config::Config, config_utils, error::SnapError, gc::{GarbageCollector, GarbageLimit}, registry::BackupEntry, snapshot::Snapshot}};

pub fn backup_data(src: &Path, dest: &Path, comp: Option<String>, config: Option<Config>) -> Result<(), SnapError> {
    let password = utils::read_password()?;

    let mut registry = utils::get_registry();
    let entry = registry.find_entry(src.to_path_buf(), dest.to_path_buf());

    let validated_password = if let Some(ent) = entry {
        let prev_password = &ent.password;
        if let Err(err) =  prev_password.verify(&password) {
            return Err(SnapError::Password(err));
        } else {
            prev_password
        }
    }else {
        &Password::new(password, &PasswordPolicy::default())?
    };

    let (algorithm, config) = confirm_algorithm(comp, config);

    if !dest.exists() {
        fs::create_dir_all(&dest)?;
    }

    let blobs_dir = dest.join("blobs");
    let snapshot_dir = dest.join("snapshot");

    fs::create_dir_all(&blobs_dir)?;
    fs::create_dir_all(&snapshot_dir)?;

    let mut garbage_info = if let Some(gl) = GarbageLimit::from_json_to_gc().ok(){
        gl
    }else {
        GarbageLimit::new()
    };

    let gc_limit = config.unwrap().general.gc_limit;

    let gc_path = blobs_dir.to_string_lossy().to_string();
    let get_gc = garbage_info.get_gc_from_limit(gc_path);
    

    let mut gc = if let Some(gc) = get_gc {
        gc.clone()
    } else {
        GarbageCollector::new(blobs_dir.clone(), gc_limit)
    };



    // we need the latest json file if there is any
    let latest_json = 
            utils::get_nth_recent_json_snapshot(0, &snapshot_dir)?
                .map(|path| std::path::PathBuf::from(path));

    let salt = utils::get_salt(&dest);
    let key = crypto::derive_key(&validated_password.hash, &salt);
    let (engine, compression) = utils::generate_compression_engine(algorithm);
    let snap = Snapshot::create(src, &blobs_dir, &key, latest_json.as_ref(), engine)?;
    
    let _ = snap.save(&snapshot_dir, &mut gc)?;
    let _ = garbage_info.add_garbage_collector_to_limit(gc);
    let _ = garbage_info.save();

    let entry = registry.find_entry(src.to_path_buf(), dest.to_path_buf());

    let ent;
    
    if let Some(en) = entry {
        let mut en = en.clone();
        en.add_snapshot();
        ent = en.clone();
    }
    else {
        ent = BackupEntry::new(snap.timestamp, src.to_path_buf(), dest.to_path_buf(), &validated_password, compression);
    }

    let _ = registry.add_backup(ent);
    let _ = registry.save_to_file();

    println!("Backup completed successfully");

    Ok(())
}

/// Given an algorithm and config, if the algorithm is `None` and the config is `None`, 
/// build a global config and assign the compression on the config to the algorithm
/// if only algorithm is `None`, assign the config's compression algorithm to `algorithm`
pub fn confirm_algorithm(algorithm: Option<String>, config: Option<Config>) -> (Option<String>, Option<Config>) {
    let mut algo = algorithm.clone();
    let mut conf = config.clone();

    if algorithm.is_none() && config.is_none() {
        println!("No config has been defined yet. \nWe will require you to build a Global one. \nPlease respond to the prompts...");
        conf = config_utils::build_global_config().ok();
        let general_conf = conf.as_ref().unwrap().general.clone();
        algo = Some(general_conf.compression);
    }

    if algorithm.is_none() {
        let general_conf = &conf.as_ref().unwrap().general;
        algo = Some(general_conf.compression.clone());
    }

    (algo, conf)
}
