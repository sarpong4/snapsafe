use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

mod common;
use common::{compare_dirs, get_password, setup_file_dirs, setup_dir, write_test_file, clear_test_registry};
use tempfile::tempdir;

fn backup_n_times(n: usize, source: PathBuf, dest: PathBuf) -> (PathBuf, PathBuf) {
    for i in 0..n {
        if i > 0 {
            let file_path = source.join(format!("file_{}.txt", i));
            write_test_file(file_path, "Adding a new file with content");
        }

        let mut cmd = Command::cargo_bin("snapsafe").unwrap();
        cmd.env("SNAPSAFE_PASSWORD", get_password())
            .env("SNAPSAFE_TEST_REGISTRY", "yes")
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
fn test_cli_backup_with_password() {
    let (source, dest) = setup_file_dirs();

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("backup")
        .arg("--source")
        .arg(source)
        .arg("--dest")
        .arg(dest);

    cmd.assert().success().stdout(contains("Backup completed successfully"));

    clear_test_registry();
}

#[test]
fn test_cli_backup_without_source_directory() {
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("backup")
        .arg("--dest")
        .arg("/backup_target");

    cmd.assert()
        .failure()
        .stderr(contains("the following required arguments were not provided"));

    clear_test_registry();
}

#[test]
fn test_cli_backup_without_target_directory() {
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("backup")
        .arg("--source")
        .arg("/test_backup");

    cmd.assert()
        .failure()
        .stderr(contains("the following required arguments were not provided"));

    clear_test_registry();
}

#[test]
fn test_cli_restore_with_correct_password_but_no_data_has_been_backed_up_should_fail() {
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    let origin = tempdir().unwrap();
    let output = tempdir().unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("restore")
        .arg("--origin")
        .arg(origin.path())
        .arg("--output")
        .arg(output.path());

    cmd.assert()
        .failure()
        .stderr(contains("No data backup available"));

    clear_test_registry();
}

#[test]
fn test_cli_restore_with_incorrect_password_treated_as_no_backup_data() {
    let (source, dest) = setup_file_dirs();

    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("backup")
        .arg("--source")
        .arg(&source)
        .arg("--dest")
        .arg(&dest);

    cmd.assert().success().stdout(contains("Backup completed successfully"));

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();

    cmd2.env("SNAPSAFE_PASSWORD", "wrong password")
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("restore")
        .arg("--origin")
        .arg(&dest)
        .arg("--output")
        .arg(&source);

    cmd2.assert()
        .failure()
        .stderr(contains("No data backup available"));

    clear_test_registry();
}

#[test]
fn test_cli_restore_with_correct_password_after_successful_backup() {
    let (source, dest) = setup_file_dirs();
    let restore_dest = setup_dir();
    
    let (_, dest1) = backup_n_times(1, source.clone(), dest);

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();

    cmd2.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("restore")
        .arg("--number")
        .arg("1")
        .arg("--origin")
        .arg(&dest1)
        .arg("--output")
        .arg(&restore_dest);

    cmd2
        .assert()
        .success()
        .stdout(contains(format!("Restore to {} completed.", restore_dest.display())));

    assert!(compare_dirs(source, restore_dest).unwrap());

    clear_test_registry();
}

#[test]
fn test_cli_restore_3rd_version_after_one_backup_treated_as_no_backup_data() {
    let (source, dest) = setup_file_dirs();
    let restore_dest = setup_dir();
    
    let (_, dest1) = backup_n_times(1, source, dest);

    let mut cmd2 = Command::cargo_bin("snapsafe").unwrap();

    cmd2.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("restore")
        .arg("--number")
        .arg("3")
        .arg("--origin")
        .arg(&dest1)
        .arg("--output")
        .arg(&restore_dest);

    cmd2.assert()
        .failure()
        .stderr(contains("No data backup available"));

    clear_test_registry();
}

#[test]
fn test_cli_delete_with_no_backup_data_should_fail() {
    let origin = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .env("SNAPSAFE_TEST_REGISTRY", "yes")
        .arg("delete")
        .arg("--origin")
        .arg(origin.path())
        .arg("--force");

    cmd.assert()
        .failure();

    clear_test_registry();
}

#[test]
fn test_cli_delete_with_incorrect_password_should_fail() {}

#[test]
fn test_cli_delete_after_one_backup_should_pass() {}

#[test]
fn test_cli_delete_1st_version_after_3_backups_should_succeed() {}

#[test]
fn test_cli_delete_3rd_version_after_1_backup_should_fail() {}

#[test]
fn test_cli_list_with_no_backups_should_print_nothing() {}

#[test]
fn test_cli_list_after_backup_should_print_information() {}

#[test]
fn test_cli_list_after_backup_and_delete_should_print_nothing() {}

#[test]
fn test_cli_list_after_backup_and_restore_should_print_nothing() {}
