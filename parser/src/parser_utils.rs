use nom::{
    error::{Error, ErrorKind},
    Err, Needed,
};

use crate::Id;

// TODO: turn into a loop
pub fn count_leading_zero_bits(input: u8) -> u8 {
    if input & 0b10000000 != 0 {
        0
    } else if input == 0 {
        8
    } else {
        count_leading_zero_bits(input << 1) + 1
    }
}

pub fn check_id_matches(input: &[u8], id: Id, expected_id: Id) -> Result<(), Err<Error<&[u8]>>> {
    if id != expected_id {
        Err(Err::Failure(Error::new(input, ErrorKind::Tag)))
    } else {
        Ok(())
    }
}

pub fn check_input_buffer_is_big_enough(input: &[u8], size: u64) -> Result<(), Err<Error<&[u8]>>> {
    if input.len() < size as usize {
        Err(Err::Incomplete(Needed::Size(
            (size as usize - input.len()).try_into().unwrap(),
        )))
    } else {
        Ok(())
    }
}
