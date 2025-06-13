pub mod actions;
pub mod commands;
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

    use crate::utils::gc::GarbageCollector;


    #[test]
    fn test_garbage_collector_prunes_old_versions() {
        let blobs_dir = tempdir().unwrap();
        let blobs_dir = blobs_dir.path();
        let mut gc = GarbageCollector::new(blobs_dir.to_path_buf(), 3);

        let path = "dir/file.rs";
        let hashes = ["h1", "h2", "h3", "h4"];
        
        for h in hashes {
            File::create(blobs_dir.join(h)).unwrap();
            gc.register_file(&PathBuf::from(path), h).unwrap();
        }

        let current = gc.get_index().get(path).unwrap();
        assert_eq!(current.len(), 3);
        assert_eq!(current, &["h4", "h3", "h2"]);
        assert!(!blobs_dir.join("h1").exists());
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
            gc.register_file(&PathBuf::from(path), h).unwrap();
        }

        let current = gc.get_index().get(path).unwrap();
        assert_eq!(current.len(), 1);
        assert_eq!(current, &["h1"]);
    }
}
