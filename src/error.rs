use std::string::FromUtf8Error;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("parsing error")]
    Parsing(#[from] nom::Err<()>),
    #[error("invalid id")]
    InvalidId,
    #[error("invalid varint")]
    InvalidVarint,
    #[error("forbidden unknown size")]
    ForbiddenUnknownSize,
    #[error("{0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("forbidden integer size")]
    ForbiddenIntegerSize,
    #[error("forbidden float size")]
    ForbiddenFloatSize,
    #[error("valid element not found")]
    ValidElementNotFound,
    #[error("missing track number")]
    MissingTrackNumber,
}
