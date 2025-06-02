use assert_cmd::Command;
use predicates::str::contains;

fn get_password() -> String {
    String::from("password")
}

#[test]
fn test_cli_backup_with_password() {
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();

    cmd.env("SNAPSAFE_PASSWORD", get_password())
        .arg("backup")
        .arg("--source")
        .arg("/test_backup")
        .arg("--dest")
        .arg("/backup_target");

    cmd.assert().success().stdout(contains("Backup completed successfully"));
}