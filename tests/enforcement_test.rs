use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

mod common;
use common::{get_password, setup_file_dirs, write_test_file, clear_test_registry};

use crate::common::get_test_registry;

fn backup_n_times(n: usize, source: PathBuf, dest: PathBuf, registry: String) -> (PathBuf, PathBuf) {
    for i in 0..n {
        if i > 0 {
            let file_path = source.join(format!("file_{}.txt", i));
            write_test_file(file_path, "Adding a new file with content");
        }

        let mut cmd = Command::cargo_bin("snapsafe").unwrap();
        cmd.env("SNAPSAFE_PASSWORD", get_password())
            .env("SNAPSAFE_TEST_REGISTRY", &registry)
            .env("TEST_CONFIG", &registry)
            .arg("backup")
            .arg("--source")
            .arg(&source)
            .arg("--dest")
            .arg(&dest);

        cmd.assert().success();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    (source, dest)
}

#[test]
fn test_cli_backup_ensures_strict_password_enforcement() {
    let registry = get_test_registry();
    let (source, dest) = setup_file_dirs();
    
    let (source1, dest1) = backup_n_times(1, source.clone(), dest, registry.clone());

    let file_path = source.join("file1.txt");
    write_test_file(file_path, "Adding a new file with content");

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    cmd.env("SNAPSAFE_PASSWORD", "wrong password")
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("backup")
        .arg("--source")
        .arg(source1)
        .arg("--dest")
        .arg(dest1);

    cmd.assert()
        .failure()
        .stderr(contains("Destination password is different from the password you provided."));

    clear_test_registry(&registry);
}