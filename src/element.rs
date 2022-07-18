use crate::Id;

/// A wrapper around an object that represents a WebM element.
///
/// Since some elements may be absent, this wrapper is used to indicate the
/// presence (or lack thereof) of an element in a WebM document. If the element is
/// encoded in the file and it has been parsed, `is_present()` will return true.
/// Otherwise it will return false since the element was ommitted or skipped when
/// parsing.
#[derive(Debug, PartialEq)]
pub struct Element<T> {
    value: T,
    is_present: bool,
}

impl<T> Element<T> {
    pub const fn new(value: T, is_present: bool) -> Self {
        Self { value, is_present }
    }

    pub const fn value(&self) -> &T {
        &self.value
    }

    pub fn mut_value(&mut self) -> &mut T {
        &mut self.value
    }

    pub const fn is_present(&self) -> bool {
        self.is_present
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment() {
        let mut e = Element::<i32>::new(42, true);
        assert!(e.is_present());
        assert_eq!(*e.mut_value(), 42);

        *e.mut_value() = 0;
        assert!(e.is_present());
        assert_eq!(*e.value(), 0);
    }
}
