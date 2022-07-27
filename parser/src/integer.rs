use nom::{bytes::streaming::take, IResult};

use crate::{element::Element, element_metadata::parse_element_metadata, ElementMetadata};

pub trait Integer64FromBigEndianBytes {
    fn from_be_bytes(input: [u8; 8]) -> Self;
}

impl Integer64FromBigEndianBytes for u64 {
    fn from_be_bytes(input: [u8; 8]) -> Self {
        u64::from_be_bytes(input)
    }
}

impl Integer64FromBigEndianBytes for i64 {
    fn from_be_bytes(input: [u8; 8]) -> Self {
        i64::from_be_bytes(input)
    }
}

pub type UnsignedElement = Element<u64>;
pub type SignedElement = Element<i64>;

pub fn parse_int<T: Integer64FromBigEndianBytes>(input: &[u8]) -> IResult<&[u8], Element<T>> {
    let (input, metadata) = parse_element_metadata(input)?;

    let (input, int_bytes) = take(metadata.size)(input)?;
    // any efficient way to avoid this copy here?
    let mut value_buffer = [0u8; 8];
    value_buffer[(8 - int_bytes.len())..].copy_from_slice(int_bytes);
    let value = T::from_be_bytes(value_buffer);

    Ok((input, Element { value, metadata }))
}

#[cfg(test)]
mod tests {
    use crate::Id;

    use super::*;

    #[test]
    fn test_parse_int() {
        const EMPTY: &[u8] = &[];
        assert_eq!(
            parse_int(&[0x42, 0x86, 0x81, 0x01]),
            Ok((
                EMPTY,
                Element {
                    value: 1u64,
                    metadata: ElementMetadata {
                        id: Id::EbmlVersion,
                        header_size: 3,
                        size: 1
                    }
                }
            ))
        )
    }
}
