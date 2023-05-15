use mkvdump::parse_elements_from_file;
use mkvparser::{elements::Id, parse_element};

const BUFFER_SIZE: u64 = 64 * 1024 * 1024;

#[test]
fn test_show_position() -> anyhow::Result<()> {
    const INPUT: &[u8] = include_bytes!("inputs/matroska-test-suite/test7.mkv");
    let elements = parse_elements_from_file(
        "tests/inputs/matroska-test-suite/test7.mkv",
        true,
        BUFFER_SIZE,
    )?;
    for element in elements {
        // Corrupted elements won't match as we ignore their ID due to invalid content.
        if element.header.id == Id::Corrupted {
            continue;
        }
        let (_, element_at_position) =
            parse_element(&INPUT[element.header.position.unwrap()..]).unwrap();
        assert_eq!(element_at_position.header.id, element.header.id);
    }
    Ok(())
}
