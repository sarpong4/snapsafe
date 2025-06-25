use std::{fs, io, path::Path};

use crate::{actions::crypto, config::{self, configs::Config}, utils::{self, gc::{GarbageCollector, GarbageLimit}, registry::BackupEntry, snapshot::Snapshot, SnapError}};

pub fn backup_data(src: &Path, dest: &Path, comp: Option<String>, config: Option<Config>) -> io::Result<()> {
    let password = utils::read_password();
    let mut algorithm = comp;
    let mut config = config;

    if algorithm.is_none() && config.is_none() {
        // let us build a config
        println!("No config has been defined yet. \nWe will require you to build a Global one. \nPlease respond to the prompts...");
        config = Some(config::build_global_config()?);
        let general_conf = config.as_ref().unwrap().general.clone();
        algorithm = Some(general_conf.compression);
        
    }

    if algorithm.is_none() {
        let general_conf = config.as_ref().unwrap().general.clone();
        algorithm = Some(general_conf.compression);
    }

    if !dest.exists() {
        fs::create_dir_all(&dest)?;
    }

    let blobs_dir = dest.join("blobs");
    let snapshot_dir = dest.join("snapshot");

    fs::create_dir_all(&blobs_dir)?;
    fs::create_dir_all(&snapshot_dir)?;

    let mut registry = utils::get_registry();
    let entry = registry.find_entry(src.to_path_buf(), dest.to_path_buf());

    let password_hash = utils::hash_password(&password);
    let comp_from_entry = utils::compare_password(entry, &password_hash, SnapError::Backup);

    if let Err(err) = comp_from_entry {
        return Err(err);
    }
    else {
        let coms = comp_from_entry.unwrap();
        if let Some(_) = coms {
            algorithm = coms;
        }
    }

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
    let key = crypto::derive_key(&password, &salt);
    let (engine, compression) = utils::generate_compression_engine(algorithm);
    let snap = Snapshot::create(src, &blobs_dir, &key, latest_json.as_ref(), engine);
    
    if let Err(err) = snap {
        eprintln!("Backup Aborted!");
        return Err(err);
    }
    let snap = snap?;
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
        ent = BackupEntry::new(snap.timestamp, src.to_path_buf(), dest.to_path_buf(), password_hash, compression);
    }

    let _ = registry.add_backup(ent);
    let _ = registry.save_to_file();

    println!("Backup completed successfully");

    Ok(())
}