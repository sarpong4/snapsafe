use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

mod common;
use common::{compare_dirs, get_password, get_test_registry, setup_file_dirs, setup_dir, write_test_file, clear_test_registry};
use tempfile::tempdir;

use crate::common::copy_dir_contents;

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

// BACKUP COMMAND TESTS

#[test]
fn test_cli_backup_with_password() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("backup")
        .arg("--source")
        .arg(source)
        .arg("--dest")
        .arg(dest);

    let assert = cmd.assert();

    clear_test_registry(&registry);
    assert.success().stdout(contains("Backup completed successfully"));
}

#[test]
fn test_cli_backup_without_source_directory() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("backup")
        .arg("--dest")
        .arg("/backup_target");

    let assert = cmd.assert();

    clear_test_registry(&registry);
    assert.failure()
        .stderr(contains("the following required arguments were not provided"));
}

#[test]
fn test_cli_backup_without_target_directory() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("backup")
        .arg("--source")
        .arg("/test_backup");

    let assert = cmd.assert();

    clear_test_registry(&registry);
    assert.failure()
        .stderr(contains("the following required arguments were not provided"));
}

// RESTORE COMMAND TESTS

#[test]
fn test_cli_restore_with_correct_password_but_no_data_has_been_backed_up_should_fail() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    let origin = tempdir().unwrap();
    let output = tempdir().unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("restore")
        .arg("--origin")
        .arg(origin.path())
        .arg("--output")
        .arg(output.path());

    let assert = cmd.assert();

    clear_test_registry(&registry);
    assert.failure()
        .stderr(contains("No backup available at path"));
}

#[test]
fn test_cli_restore_with_incorrect_password_treated_as_no_backup_data() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("backup")
        .arg("--source")
        .arg(&source)
        .arg("--dest")
        .arg(&dest);

    cmd.assert().success().stdout(contains("Backup completed successfully"));

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();

    cmd2.env("SNAPSAFE_PASSWORD", "Wrong2Password;")
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("restore")
        .arg("--origin")
        .arg(&dest)
        .arg("--output")
        .arg(&source);

    let assert = cmd2.assert();

    clear_test_registry(&registry);
    assert.failure()
        .stderr(contains("Password Error: IncorrectPassword"));
}

#[test]
fn test_cli_restore_with_correct_password_after_successful_backup() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let restore_dest = setup_dir();
    
    let (_, dest1) = backup_n_times(1, source.clone(), dest, registry.clone());

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();

    cmd2.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("restore")
        .arg("--number")
        .arg("1")
        .arg("--origin")
        .arg(&dest1)
        .arg("--output")
        .arg(&restore_dest);

    let assert = cmd2.assert();

    assert!(compare_dirs(source, restore_dest.clone()).unwrap());

    clear_test_registry(&registry);
    assert.success()
        .stdout(contains(format!("Restore to {:?} completed.", restore_dest.as_path().display())));
}

#[test]
fn test_cli_restore_3rd_version_after_one_backup_treated_as_no_backup_data() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let restore_dest = setup_dir();
    
    let (_, dest1) = backup_n_times(1, source, dest, registry.clone());

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();

    cmd2.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("restore")
        .arg("--number")
        .arg("3")
        .arg("--origin")
        .arg(&dest1)
        .arg("--output")
        .arg(&restore_dest);

    let assert = cmd2.assert();

    clear_test_registry(&registry);
    assert.failure()
        .stderr(contains("Failed to restore"));
}

// DELETE COMMAND TESTS

#[test]
fn test_cli_delete_with_no_backup_data_should_fail() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let origin = tempdir().unwrap();
    let origin = origin.path();
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("delete")
        .arg("--origin")
        .arg(origin)
        .arg("--force");

    let assert = cmd.assert();

    clear_test_registry(&registry);
    assert.failure()
        .stderr(contains("Target provided does not exist"));
}

#[test]
fn test_cli_delete_with_incorrect_password_should_fail() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let (_source, dest) = backup_n_times(1, source.clone(), dest.clone(), registry.clone());

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", "wrong_password")
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .arg("delete")
        .env("TEST_CONFIG", &registry)
        .arg("--origin")
        .arg(dest)
        .arg("--force");

    let assert = cmd.assert();

    clear_test_registry(&registry);

    assert.failure()
        .stderr(contains("Password Error: IncorrectPassword"));
}

#[test]
fn test_cli_delete_after_one_backup_should_pass() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let (_source, dest) = backup_n_times(1, source.clone(), dest.clone(), registry.clone());

    let dest_after_backup = tempdir().unwrap();
    let dest_after_backup = dest_after_backup.path();

    let _ = copy_dir_contents(&dest, &dest_after_backup);

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("delete")
        .arg("--origin")
        .arg(&dest)
        .arg("--force");

    let assert = cmd.assert();

    assert!(!compare_dirs(dest_after_backup.to_path_buf(), dest).unwrap());

    clear_test_registry(&registry);

    assert.success()
        .stdout(contains("Deletion complete."));
}

#[test]
fn test_cli_delete_1st_version_after_3_backups_should_succeed() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let (_source, dest) = backup_n_times(3, source.clone(), dest.clone(), registry.clone());

    let dest_after_backup = tempdir().unwrap();

    let dest_after_backup = dest_after_backup.path();

    let _ = copy_dir_contents(&dest, &dest_after_backup);

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("delete")
        .arg("--number")
        .arg("3")
        .arg("--origin")
        .arg(&dest)
        .arg("--force");

    let assert = cmd.assert();

    assert!(!compare_dirs(dest_after_backup.to_path_buf(), dest).unwrap());

    clear_test_registry(&registry);

    assert.success()
        .stdout(contains("Deletion complete."));
}

#[test]
fn test_cli_delete_3rd_version_after_1_backup_should_fail() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let (_source, dest) = backup_n_times(1, source.clone(), dest.clone(), registry.clone());

    let dest_after_backup = tempdir().unwrap();

    let dest_after_backup = dest_after_backup.path();

    let _ = copy_dir_contents(&dest, &dest_after_backup);

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("delete")
        .arg("--number")
        .arg("3")
        .arg("--origin")
        .arg(&dest)
        .arg("--force");

    let assert = cmd.assert();

    clear_test_registry(&registry);

    assert.failure()
        .stderr(contains("Failed to delete backup"));

}

// LIST COMMAND TESTS

#[test]
fn test_cli_list_with_no_backups_should_print_nothing() {
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    let registry = get_test_registry();

    cmd.env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry).arg("list");

    cmd.assert().success().stdout(contains("No data has been backed up"));

    clear_test_registry(&registry);
}

#[test]
fn test_cli_list_after_backup_should_print_information() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let (_, _) = backup_n_times(1, source.clone(), dest.clone(), registry.clone());

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    cmd.env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry).arg("list");

    let assert = cmd.assert();

    clear_test_registry(&registry);

    assert.success().stdout(contains("Snapshots: 1"));
}

#[test]
fn test_cli_list_after_backup_and_delete_should_print_nothing() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let (source, dest) = setup_file_dirs();
    let (_, dest1) = backup_n_times(1, source.clone(), dest.clone(), registry.clone());

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("delete")
        .arg("--origin")
        .arg(dest1)
        .arg("--force");

    cmd.assert().success();

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();
    cmd2.env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry).arg("list");

    let assert = cmd2.assert();

    clear_test_registry(&registry);
    assert.success().stdout(contains("No data has been backed up"));
}

#[test]
fn test_cli_list_after_backup_and_restore_should_print_nothing() {
    let registry = get_test_registry();
    clear_test_registry(&registry);

    let restore_dest = tempdir().unwrap();
    let restore_dest = restore_dest.path();

    let (source, dest) = setup_file_dirs();
    let (_, _) = backup_n_times(1, source.clone(), dest.clone(), registry.clone());


    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry)
        .arg("restore")
        .arg("--origin")
        .arg(&dest)
        .arg("--output")
        .arg(restore_dest);

    cmd.assert().success();

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();
    cmd2.env("SNAPSAFE_TEST_REGISTRY", &registry)
        .env("TEST_CONFIG", &registry).arg("list");

    let assert = cmd2.assert();

    clear_test_registry(&registry);
    assert.success().stdout(contains("No data has been backed up"));
}
