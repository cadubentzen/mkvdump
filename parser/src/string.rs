use nom::{
    bytes::streaming::take,
    error::{Error, ErrorKind},
    Err, IResult,
};

use crate::{element::Element, element_metadata::parse_element_metadata, ElementMetadata};

pub type StringElement = Element<String>;

pub fn parse_string(input: &[u8]) -> IResult<&[u8], StringElement> {
    let (input, metadata) = parse_element_metadata(input)?;
    let (input, string_bytes) = take(metadata.size)(input)?;
    // TODO: remove this unwrap here
    let value = String::from_utf8(string_bytes.to_vec())
        .map_err(|_| Err::Failure(Error::new(input, ErrorKind::Fail)))?;

    Ok((input, Element { value, metadata }))
}

#[cfg(test)]
mod tests {
    use crate::Id;

    use super::*;

    #[test]
    fn test_parse_string() {
        const EMPTY: &[u8] = &[];
        assert_eq!(
            parse_string(&[0x42, 0x82, 0x84, 0x77, 0x65, 0x62, 0x6D]),
            Ok((
                EMPTY,
                Element {
                    value: "webm".to_string(),
                    metadata: ElementMetadata {
                        id: Id::DocType,
                        header_size: 3,
                        size: 4
                    }
                }
            ))
        )
    }
}
