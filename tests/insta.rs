use mkvdump::{parse_element_or_skip_corrupted, tree::build_element_trees, Body, Element};

// TODO: decide where to place this helper. Currently duplicated.
fn parse_elements(input: &[u8], show_position: bool) -> Vec<Element> {
    let mut elements = Vec::<Element>::new();
    let mut read_buffer = input;
    let mut position = show_position.then_some(0);

    while let Ok((new_read_buffer, mut element)) = parse_element_or_skip_corrupted(read_buffer) {
        element.header.position = position;
        position = position.map(|p| {
            if let Body::Master = element.body {
                p + element.header.header_size
            } else {
                // It's safe to unwrap because all non-Master elements have a set size
                p + element.header.size.unwrap()
            }
        });
        elements.push(element);
        if new_read_buffer.is_empty() {
            break;
        }
        read_buffer = new_read_buffer;
    }
    elements
}

macro_rules! snapshot_test {
    ($test_name:ident, $filename:expr) => {
        #[test]
        fn $test_name() {
            let elements = parse_elements(include_bytes!($filename), false);
            insta::assert_yaml_snapshot!(build_element_trees(&elements));
        }
    };
}

snapshot_test!(
    test_parse_incomplete_file_should_not_panic,
    "../inputs/incomplete.hdr"
);
snapshot_test!(test_parse_header_encrypted, "../inputs/encrypted.hdr");

// File was generated with:
// ffmpeg -f lavfi -i testsrc -c:v libx264 -frames:v 2 -metadata creation_time="2022-08-11T08:27:15Z" -f matroska test.mkv
snapshot_test!(test_parse_file_with_dateutc, "../inputs/dateutc.mkv");

// Tests from Matroska test suite
snapshot_test!(test1, "../inputs/matroska-test-suite/test1.mkv");
snapshot_test!(test2, "../inputs/matroska-test-suite/test2.mkv");
snapshot_test!(test3, "../inputs/matroska-test-suite/test3.mkv");
snapshot_test!(test4, "../inputs/matroska-test-suite/test4.mkv");
snapshot_test!(test5, "../inputs/matroska-test-suite/test5.mkv");
snapshot_test!(test6, "../inputs/matroska-test-suite/test6.mkv");
snapshot_test!(test7, "../inputs/matroska-test-suite/test7.mkv");
snapshot_test!(test8, "../inputs/matroska-test-suite/test8.mkv");

snapshot_test!(
    test_two_inits_segment_unknown_size,
    "../inputs/two_inits_segment_unknown_size.webm"
);
snapshot_test!(
    test_init_after_cluster_unknown_size,
    "../inputs/init_after_cluster_unknown_size.webm"
);
