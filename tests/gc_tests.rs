use std::{fs::File, path::PathBuf};

use tempfile::tempdir;

use snapsafe::actions::{gc::GarbageCollector};

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
