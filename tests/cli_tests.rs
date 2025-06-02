use assert_cmd::Command;

fn get_password() -> String {
    String::from("password")
}

#[test]
fn test_cli_backup_with_password() {
    let mut cmd = Command::cargo_bin("snapsafe").unwrap();
    cmd.arg("backup")
        .arg("--source")
        .arg("/test_backup")
        .arg("--dest")
        .arg("/backup_target");

    let password = get_password();

    cmd.assert().success();
}