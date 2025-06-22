use std::{fs, io, path::Path};

use crate::{actions::crypto, config::{build_global_config, Config}, utils::{self, registry::BackupEntry, snapshot::Snapshot}};

pub fn backup_data(src: &Path, dest: &Path, comp: Option<String>, config: Option<Config>) -> io::Result<()> {
    let password = utils::read_password();
    let mut algorithm = comp;

    if config.is_none() {
        // let us build a config
        println!("No config has been defined yet. We will require you to build one. Please respond to the prompts");
        let config = Some(build_global_config()?);
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

    // strict no password change enforcement.
    if let Some(ent) = entry {
        if password_hash != ent.passsword_hash {
            eprintln!("Backup destination already initialized with a different password. You cannot backup to this destination with a different password");
            let err = utils::get_error(utils::SnapError::Backup);
            return Err(err);
        }

        // use the compression algorithm on the BackupEntry even if it is different from the one 
        // the user provided now. We use the same algorithm throughout when we compress a 
        // given source -> destination
        algorithm = Some(ent.compression_algorithm.clone()); 
    };



    // we need the latest json file if there is any
    let latest_json = 
            utils::get_nth_recent_json_snapshot(0, &snapshot_dir)?
                .map(|path| std::path::PathBuf::from(path));

    let salt = utils::get_salt(&dest);
    let key = crypto::derive_key(&password, &salt);
    let (engine, conf) = utils::generate_compression_engine(algorithm);
    let snap = Snapshot::create(src, &blobs_dir, &key, latest_json.as_ref(), engine);
    
    if let Err(err) = snap {
        eprintln!("Backup Aborted!");
        return Err(err);
    }
    let snap = snap?;
    let _ = snap.save(&blobs_dir, &snapshot_dir)?;

    let entry = registry.find_entry(src.to_path_buf(), dest.to_path_buf());

    let ent;
    
    if let Some(en) = entry {
        let mut en = en.clone();
        en.add_snapshot();
        ent = en.clone();
    }
    else {
        ent = BackupEntry::new(snap.timestamp, src.to_path_buf(), dest.to_path_buf(), password_hash, conf);
    }

    let _ = registry.add_backup(ent);
    let _ = registry.save_to_file();

    println!("Backup completed successfully");

    Ok(())
}