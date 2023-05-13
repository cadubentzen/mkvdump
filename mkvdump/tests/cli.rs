use assert_cmd::Command;

fn command() -> Command {
    Command::cargo_bin("mkvdump").unwrap()
}

#[test]
fn basic() {
    const INPUT: &str = "tests/inputs/encrypted.hdr";
    command().arg(INPUT).assert().success();
    command()
        .arg("-f")
        .arg("json")
        .arg(INPUT)
        .assert()
        .success();
    command()
        .arg("-f")
        .arg("yaml")
        .arg(INPUT)
        .assert()
        .success();
    command().arg("-p").arg(INPUT).assert().success();
    command().arg("-l").arg(INPUT).assert().success();
}
