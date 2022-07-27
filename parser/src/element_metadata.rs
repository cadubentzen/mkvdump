use nom::{combinator::peek, IResult};

use crate::{id::parse_id, parser_utils::check_id_matches, varint::parse_varint, Id};

/// Metadata for WebM elements that are encountered when parsing.
#[derive(Debug, PartialEq)]
pub struct ElementMetadata {
    /// The EBML ID of the element.
    pub id: Id,
    /// The number of bytes that were used to encode the EBML ID and element size.
    /// If the size of the header is unknown (which is only the case if a seek was
    /// performed to the middle of an element, so its header was not parsed), this
    /// will be None.
    pub header_size: usize,
    /// The size of the element.
    /// This is number of bytes in the element's body, which excludes the header bytes.
    /// If the size of the element's body is unknown, this will be None.
    pub size: u64,
}

pub fn parse_element_metadata(input: &[u8]) -> IResult<&[u8], ElementMetadata> {
    let initial_len = input.len();
    let (input, id) = parse_id(input)?;
    let (input, size) = parse_varint(input)?;

    let header_size = initial_len - input.len();

    Ok((
        input,
        ElementMetadata {
            id,
            header_size,
            size,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_element_metadata() {
        const EMPTY: &[u8] = &[];
        const INPUT: &[u8] = &[0x1A, 0x45, 0xDF, 0xA3, 0x9F];
        assert_eq!(
            parse_element_metadata(INPUT),
            Ok((
                EMPTY,
                ElementMetadata {
                    id: Id::Ebml,
                    header_size: 5,
                    size: 31
                }
            ))
        );
    }
}
