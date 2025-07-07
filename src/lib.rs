pub mod actions;
pub mod commands;
pub mod compress;
pub mod utils;
pub mod crypto;

#[cfg(test)]
mod crypto_tests {
    use crate::crypto::*;

    #[test]
    fn test_key_derivation_consistency() {
        let password = "my-password";
        let salt = b"fixed-salt";
        let key1 = derive_key(password, salt);
        let key2 = derive_key(password, salt);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_key_derivation_changes() {
        let password1 = "password1";
        let password2 = "password2";
        let salt = b"fixed-salt";
        let key1 = derive_key(password1, salt);
        let key2 = derive_key(password2, salt);

        assert_ne!(key1, key2);
    }

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let data = b"important data";
        let password = "backup123";
        let salt = b"some-salt";

        let key = derive_key(password, salt);
        let (encrypted, nonce) = encrypt_file_bytes(data, &key);
        let decrypted = decrypt_file_bytes(&encrypted, &key, &nonce).unwrap_or(Vec::new());
        assert_eq!(decrypted, data);
    }

    #[test]
    #[should_panic]
    fn test_wrong_key_panics() {
        let data = b"secret";
        let key1 = derive_key("pass1", b"salt");
        let key2 = derive_key("pass2", b"salt");

        let (encrypted, nonce) = encrypt_file_bytes(data, &key1);
        let _ = decrypt_file_bytes(&encrypted, &key2, &nonce); // should panic
    }
}

#[cfg(test)]
mod gc_tests {
    use std::{fs::File, path::PathBuf};

    use tempfile::tempdir;

    use crate::utils::gc::{GarbageCollector, SnapshotReference};


    #[test]
    fn test_garbage_collector_prunes_old_versions() {
        let blobs_dir = tempdir().unwrap();
        let blobs_dir = blobs_dir.path();
        let mut gc = GarbageCollector::new(blobs_dir.to_path_buf(), 3);

        let path = tempdir().unwrap();
        let path = path.path();
        let hashes = ["h1", "h2", "h3", "h4"];
        
        for hash in &hashes {
            File::create(blobs_dir.join(hash)).unwrap();
            let _ = gc.register_file(&path.to_path_buf(), &hash, &path.to_path_buf());
        }

        let mut hashes = hashes
            .map(|v| SnapshotReference::from((v.into(), v.into())));
            
        hashes.reverse();


        let current = gc.get_index()
            .get(&path.to_string_lossy().to_string()).unwrap()
                .iter()
                .map(|f| f.hash_file.clone()).collect::<Vec<String>>();

        assert_eq!(current.len(), 3);
        let mut expected_hashes = hashes.map(|h| h.hash_file).to_vec();

        let last = expected_hashes.pop().unwrap();
        assert_eq!(current, expected_hashes);
        assert!(!blobs_dir.join(last).exists());
    }

    #[test]
    fn test_garbage_collector_ignores_already_stored_hash() {
        let blobs_dir = tempdir().unwrap();
        let blobs_dir = blobs_dir.path();

        let mut gc = GarbageCollector::new(blobs_dir.to_path_buf(), 3);

        let path = "dir/file.rs";
        let hashes = ["h1", "h1", "h1"];

        for h in hashes {
            File::create(blobs_dir.join(h)).unwrap();
            gc.register_file(&PathBuf::from(path), h, &PathBuf::from(path)).unwrap();
        }

        let current = gc.get_index().get(path).unwrap().iter().map(|f| f.hash_file.clone()).collect::<Vec<String>>();
        assert_eq!(current.len(), 1);
        assert_eq!(current, &["h1"]);
    }
}

mod registry_tests {
    use std::{fs, path::PathBuf};

    use chrono::Utc;

    use crate::{crypto::password, utils::registry::{BackupEntry, BackupRegistry}};

    fn temp_registry_path() -> String {
        let tmp = std::env::temp_dir().join(format!("snapsafe_test_{}", uuid::Uuid::new_v4()));
        tmp.to_str().unwrap().to_string()
    }

    fn sample_entry() -> BackupEntry {
        BackupEntry::new(
            Utc::now(),
            PathBuf::from("/tmp/source"),
            PathBuf::from("/tmp/backup"),
            &password::Password::default(),
            "gzip".to_string(),
        )
    }

    #[test]
    fn test_backup_entry_new_and_snapshot_count() {
        let mut entry = sample_entry();
        assert_eq!(entry.snapshot_count, 1);

        entry.add_snapshot();
        assert_eq!(entry.snapshot_count, 2);

        entry.remove_snapshot();
        assert_eq!(entry.snapshot_count, 1);
    }

    #[test]
    fn test_find_entry_from_dest_with_wrong_dest_is_none() {

        let mut registry = BackupRegistry::default();
        let entry = BackupEntry::default();
        registry.registry.push(entry);

        let wrong_dest = "backup/other_file.bak";
        let found = registry.find_entry_from_dest(wrong_dest.into());
        assert!(found.is_none());
    }

    #[test]
    fn test_find_entry_from_dest_with_right_dest_should_pass() {
        let mut registry = BackupRegistry::default();
        let entry = BackupEntry::default();
        registry.add_backup(entry);

        let dest = "target/some_file.bak";
        let found = registry.find_entry_from_dest(dest.into());
        assert!(found.is_some());
    }

    #[test]
    fn test_remove_backup_from_registry_actually_removes_that_backup() {
        let mut registry = BackupRegistry::default();
        let entry = BackupEntry::default();

        registry.registry.push(entry.clone());
        registry.remove_backup(entry);

        let dest = "backup/some_file";
        let found = registry.find_entry_from_dest(dest.into());
        assert!(found.is_none());
    }

    #[test]
    fn test_add_backup_to_registry_does_add_backup() {
        let mut registry = BackupRegistry::default();
        let entry = BackupEntry::default();
        
        registry.add_backup(entry.clone());

        assert!(registry.registry.len() == 1);

        let entry2 = BackupEntry::default();
        registry.add_backup(entry2);

        assert!(registry.registry.len() == 2);
    } 

    #[test]
    fn test_backup_registry_save_and_load() {
        let temp_path = temp_registry_path();
        let mut reg = BackupRegistry::build_test_registy(temp_path.clone());
        let entry = sample_entry();

        reg.add_backup(entry.clone());
        reg.save_to_file().unwrap();

        let loaded = BackupRegistry::load_from_file(&reg.registry_path).unwrap();
        assert_eq!(loaded.registry.len(), 1);
        assert_eq!(loaded.registry[0].id, entry.id);

        // Clean up
        let _ = fs::remove_file(&reg.registry_path);
        let _ = fs::remove_dir_all(PathBuf::from(temp_path));
    }

    #[test]
    fn test_add_backup_replaces_existing() {
        let mut reg = BackupRegistry::new();
        let mut entry = sample_entry();
        reg.add_backup(entry.clone());
        assert_eq!(reg.registry.len(), 1);

        entry.snapshot_count = 2;
        reg.add_backup(entry.clone());
        assert_eq!(reg.registry.len(), 1);
        assert_eq!(reg.registry[0].snapshot_count, 2);
    }

    #[test]
    fn test_add_backup_with_zero_snapshot_count() {
        let mut reg = BackupRegistry::new();
        let mut entry = sample_entry();
        entry.snapshot_count = 0;
        reg.add_backup(entry);
        assert_eq!(reg.registry.len(), 0);
    }
}

mod password_tests {
    use crate::crypto::password::{Password, PasswordPolicy};

    #[test]
    fn test_password_roundtrip(){
        let policy = PasswordPolicy::default();
        let password = Password::new("ItisValidP3#".into(), &policy).unwrap();

        assert!(password.verify("ItisValidP3#").unwrap())
    }

    #[test]
    fn test_invalid_format(){
        let policy = PasswordPolicy::default();

        assert!(Password::new("password".into(), &policy).is_err())
    }
}

mod config_tests {
    use crate::utils::config::{Config, GeneralConfig};


    #[test]
    fn test_config_roundtrip() {
        let general = GeneralConfig::from(("some/path".into(), "gzip".into(), 3));
        let original = Config { general };

        let toml = toml::to_string(&original).unwrap();
        let deserialized: Config = toml::from_str(&toml).unwrap();

        assert_eq!(original, deserialized);
    }
}
