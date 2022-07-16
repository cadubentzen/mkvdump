/// General status.
#[derive(PartialEq, Debug)]
pub enum GeneralStatus {
    /// The operation successfully completed.
    OkCompleted,
    //// The operation was successful but only partially completed (for example, a read that resulted in fewer bytes than requested). Contains the number of bytes read or skipped.
    OkPartial(u64),
    /// The operation would block, and should be tried again later.
    WouldBlock,
    /// More data was requested but the file has ended.
    EndOfFile,
}

/// Parsing errors.
#[derive(PartialEq, Debug)]
pub enum ErrorStatus {
    /// An element's ID is malformed.
    InvalidElementId,
    /// An element's size is malformed.
    InvalidElementSize,
    /// An unknown element has unknown size.
    IndefiniteUnknownElement,
    /// A child element overflowed the parent element's bounds.
    ElementOverflow,
    /// An element's size exceeds the system's memory limits.
    NotEnoughMemory,
    /// An element's value is illegal/malformed.
    InvalidElementValue,
    /// A recursive element was so deeply nested that exceeded the parser's limit.
    ExceededRecursionDepthLimit,
}

// Internal-only codes should not be used by users.
// Additionally, these codes should never be returned to the user; doing so
// is considered a bug.
#[derive(PartialEq, Debug)]
pub enum InternalStatus {
    SwitchToSkip,
}

/// Status information that represents success, failure, etc. for operations
/// throughout the API.
#[derive(PartialEq, Debug)]
pub enum Status {
    General(GeneralStatus),
    Error(ErrorStatus),
    Internal(InternalStatus),
}

impl Status {
    pub fn ok(&self) -> bool {
        matches!(
            self,
            Self::General(GeneralStatus::OkCompleted | GeneralStatus::OkPartial(_))
        )
    }

    pub fn completed_ok(&self) -> bool {
        matches!(self, Self::General(GeneralStatus::OkCompleted))
    }

    pub fn is_parsing_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }
}

impl From<GeneralStatus> for Status {
    fn from(status: GeneralStatus) -> Self {
        Self::General(status)
    }
}

impl From<ErrorStatus> for Status {
    fn from(status: ErrorStatus) -> Self {
        Self::Error(status)
    }
}

impl From<InternalStatus> for Status {
    fn from(status: InternalStatus) -> Self {
        Self::Internal(status)
    }
}

impl PartialEq<GeneralStatus> for Status {
    fn eq(&self, other: &GeneralStatus) -> bool {
        match self {
            Self::General(general) => general == other,
            _ => false,
        }
    }
}

impl PartialEq<ErrorStatus> for Status {
    fn eq(&self, other: &ErrorStatus) -> bool {
        match self {
            Self::Error(error) => error == other,
            _ => false,
        }
    }
}

impl PartialEq<InternalStatus> for Status {
    fn eq(&self, other: &InternalStatus) -> bool {
        match self {
            Self::Internal(internal) => internal == other,
            _ => false,
        }
    }
}
