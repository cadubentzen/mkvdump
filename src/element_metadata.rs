use crate::Id;

/// Metadata for WebM elements that are encountered when parsing.
#[derive(Debug, PartialEq)]
pub struct ElementMetadata {
    /// The EBML ID of the element.
    pub id: Id,
    /// The number of bytes that were used to encode the EBML ID and element size.
    /// If the size of the header is unknown (which is only the case if a seek was
    /// performed to the middle of an element, so its header was not parsed), this
    /// will be None.
    pub header_size: Option<u32>,
    /// The size of the element.
    /// This is number of bytes in the element's body, which excludes the header bytes.
    /// If the size of the element's body is unknown, this will be None.
    pub size: Option<u64>,
    /// The absolute byte position of the element, starting at the first byte of the
    /// element's header.
    /// If the position of the element is unknown (which is only the case if a seek
    /// was performed to the middle of an element), this will be None.
    pub position: Option<u64>,
}
