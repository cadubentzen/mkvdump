use assert_cmd::Command;

#[test]
fn basic() {
    let mut cmd = Command::cargo_bin("mkvdump").unwrap();
    cmd.arg("--help").assert().success();
}
