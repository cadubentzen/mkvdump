use mkvdump::parse_elements_from_file;

#[test]
fn given_small_buffer_fail_when_element_does_not_fit() {
    assert_eq!(
        parse_elements_from_file("tests/inputs/matroska-test-suite/test1.mkv", false, 65536)
            .unwrap_err()
            .to_string(),
        "failed to parse the given file with buffer size of 65536 bytes"
    );
}
