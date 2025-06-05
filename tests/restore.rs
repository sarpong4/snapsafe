// use std::{collections::HashMap, fs, path::PathBuf, time::SystemTime};

// use rand::RngCore;
// use snapsafe::actions::{crypto, snapshot};
// use tempfile::tempdir;

// #[test]
// fn test_restore_restores_encrypted_file() {
//     let dir = tempdir().unwrap();
//     let dir = dir.path();

//     let source_dir = dir.join("source");
//     let target_dir = dir.join("target");
//     let blobs_dir = source_dir.join("blobs");
//     let snapshot_dir = source_dir.join("snapshot");

//     let _ = fs::create_dir_all(&blobs_dir);
//     let _ = fs::create_dir_all(&snapshot_dir);
//     let _ = fs::create_dir_all(&target_dir);

//     let password = "test-password";
//     let salt = b"testing-salt";
//     let key = crypto::derive_key(&password, salt);
//     let plaintext = b"TESTING for the plaintext";

//     let mut nonce = [0u8; 12];
//     rand::rng().fill_bytes(&mut nonce);

//     // Encryption;
//     let (ciphertext, nonce) = crypto::encrypt_file_bytes(plaintext, &key);
//     let hash = "somefakehashforthesakeoftesting1";
//     let encrypted_file_path = blobs_dir.join(hash);
//     let _ = fs::write(&encrypted_file_path, ciphertext);


//     let mut files = HashMap::new();
//     files.insert(PathBuf::from("file.txt"), snapshot::FileEntry{
//         hash: hash.to_string(),
//         nonce,
//         modified: SystemTime::now(),
//     });

//     let snapshot = snapshot::Snapshot::new(files);
//     let snapshot_path = snapshot_dir.join("snapshot.json");
//     let _ = snapshot.to_json_file(&snapshot_path);
// }