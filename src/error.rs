use std::{num::TryFromIntError, string::FromUtf8Error};

/// An Error while parsing Matroska/WebM files
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    /// Need data
    #[error("need data")]
    NeedData,
    /// Parsing error
    #[error("parser error")]
    Parser,
    /// Invalid ID
    #[error("invalid id")]
    InvalidId,
    /// Invalid Varint
    #[error("invalid varint")]
    InvalidVarint,
    /// Unknown Element size where it's forbidden
    #[error("forbidden unknown size")]
    ForbiddenUnknownSize,
    /// Error building UTF-8 string
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    /// Forbidden Integer size
    #[error("forbidden integer size")]
    ForbiddenIntegerSize,
    /// Forbidden Float size
    #[error("forbidden float size")]
    ForbiddenFloatSize,
    /// No valid element found
    #[error("valid element not found")]
    ValidElementNotFound,
    /// Missing track number
    #[error("missing track number")]
    MissingTrackNumber,
    /// Overflow
    #[error("overflow")]
    Overflow(#[from] TryFromIntError),
    /// Invalid Date
    #[error("invalid date")]
    InvalidDate,
}

impl From<nom::Err<()>> for Error {
    fn from(value: nom::Err<()>) -> Self {
        match value {
            nom::Err::Incomplete(_) => Self::NeedData,
            _ => Self::Parser,
        }
    }
}

// FIXME(#53) This is mostly to keep coverage happy, but that error type will
// in practice never be instantiated as we don't use combinators in nom.
// After removing nom as a dependency we should be able to remove this test as well.
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parser() {
        assert_eq!(Error::Parser, nom::Err::Error(()).into());
    }
}
