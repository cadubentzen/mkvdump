use nom::{
    bytes::streaming::take,
    combinator::peek,
    error::{Error, ErrorKind},
    Err, IResult,
};

use crate::parser_utils::count_leading_zero_bits;

pub fn parse_varint(first_input: &[u8]) -> IResult<&[u8], u64> {
    let (input, first_byte) = peek(take(1usize))(first_input)?;
    let first_byte = first_byte[0];

    let vint_prefix_size = count_leading_zero_bits(first_byte) + 1;

    // Maximum 8 bytes, i.e. first byte can't be 0
    if vint_prefix_size > 8 {
        return Err(Err::Failure(Error::new(first_input, ErrorKind::Fail)));
    }

    let (input, varint_bytes) = take(vint_prefix_size)(input)?;
    // any efficient way to avoid this copy here?
    let mut value_buffer = [0u8; 8];
    value_buffer[(8 - varint_bytes.len())..].copy_from_slice(varint_bytes);
    let mut value = u64::from_be_bytes(value_buffer);

    // discard varint prefix (zeros + market bit)
    let num_bits_in_value = 7 * vint_prefix_size;
    value &= (1 << num_bits_in_value) - 1;

    Ok((input, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_varint() {
        const EMPTY: &[u8] = &[];
        assert_eq!(parse_varint(&[0x9F]), Ok((EMPTY, 31)));
        assert_eq!(parse_varint(&[0x81]), Ok((EMPTY, 1)));
        assert_eq!(parse_varint(&[0x53, 0xAC]), Ok((EMPTY, 5036)));

        const INVALID_VARINT: &[u8] = &[0x00, 0xAC];
        assert_eq!(
            parse_varint(INVALID_VARINT),
            Err(Err::Failure(Error::new(INVALID_VARINT, ErrorKind::Fail)))
        );
    }
}
