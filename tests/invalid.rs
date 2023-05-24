use mkvdump::parse_elements_from_file;

#[test]
fn should_not_hang() {
    assert!(parse_elements_from_file("tests/inputs/invalid.txt", false).is_ok());
}
