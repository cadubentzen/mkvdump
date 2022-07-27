use crate::{status::GeneralStatus, Ebml, ElementMetadata, Reader, Status};

/// The action to be performed when parsing an element.
pub enum Action {
    /// Read and parse the element.
    Read,
    /// Skip the element. Skipped elements are not parsed or stored, and the callback
    /// is not given any further notifications regarding the element.
    Skip,
}

/// A callback that receives parsing events.
///
/// Every method that returns a `Status` should return `GeneralStatus::OkCompleted` when
/// the method has completed and parsing should continue. Returning any other value
/// will cause parsing to stop. Parsing may be resumed if the returned status was
/// not a parsing error (see `Status::is_parsing_error()`). When parsing is
/// resumed, the same `Callback` method will be called again.
///
/// Methods that take a `Reader` expect the implementation to consume (either via
/// `Reader::Read()` or `Reader::Skip()`) the specified number of bytes before
/// returning `Status::kOkCompleted`. Default implementations will call
/// `Reader::Skip()` to skip the specified number of bytes and the resulting
/// `Status` will be returned (unless it's `Status::kOkPartial`, in which case
/// `Reader::Skip()` will be called again to skip more data).
///
/// Users should derive from this trait and override member methods as needed.
pub trait Callback {
    /// Called when the parser starts a new element. This is called after the
    /// elements ID and size has been parsed, but before any of its body has been
    /// read (or validated).
    ///
    /// Defaults to `Action::Read` and returning `GeneralStatus::OkCompleted`.
    ///
    /// Parameters:
    /// `metadata` - Metadata about the element that has just been encountered.
    /// `action` - The action that should be taken when handling this
    /// element. Needs to be set
    fn on_element_begin(&mut self, metadata: &ElementMetadata) -> (Status, Action) {
        (GeneralStatus::OkCompleted.into(), Action::Read)
    }

    fn on_unknown_element(
        &mut self,
        metadata: &ElementMetadata,
        reader: &mut dyn Reader,
    ) -> Status {
        todo!()
    }

    fn on_ebml(&mut self, metadata: &ElementMetadata, ebml: &Ebml) -> Status {
        todo!()
    }
}
