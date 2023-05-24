use mkvdump::parse_elements_from_file;
use mkvparser::tree::build_element_trees;

const BUFFER_SIZE: u64 = 8192;

macro_rules! snapshot_test {
    ($test_name:ident, $filename:expr) => {
        #[test]
        fn $test_name() -> anyhow::Result<()> {
            let elements =
                parse_elements_from_file(concat!("tests/inputs/", $filename), false, BUFFER_SIZE)?;
            insta::assert_yaml_snapshot!(build_element_trees(&elements));
            Ok(())
        }
    };
}

snapshot_test!(test_parse_incomplete_file_should_not_hang, "incomplete.hdr");
snapshot_test!(test_parse_header_encrypted, "encrypted.hdr");

// File was generated with:
// ffmpeg -f lavfi -i testsrc -c:v libx264 -frames:v 2 -metadata creation_time="2022-08-11T08:27:15Z" -f matroska test.mkv
snapshot_test!(test_parse_file_with_dateutc, "dateutc.mkv");

// Tests from Matroska test suite
snapshot_test!(test1, "matroska-test-suite/test1.mkv");
snapshot_test!(test2, "matroska-test-suite/test2.mkv");
snapshot_test!(test3, "matroska-test-suite/test3.mkv");
snapshot_test!(test4, "matroska-test-suite/test4.mkv");
snapshot_test!(test5, "matroska-test-suite/test5.mkv");
snapshot_test!(test6, "matroska-test-suite/test6.mkv");
snapshot_test!(test7, "matroska-test-suite/test7.mkv");
snapshot_test!(test8, "matroska-test-suite/test8.mkv");

snapshot_test!(
    test_two_inits_segment_unknown_size,
    "two_inits_segment_unknown_size.webm"
);
snapshot_test!(
    test_init_after_cluster_unknown_size,
    "init_after_cluster_unknown_size.webm"
);
