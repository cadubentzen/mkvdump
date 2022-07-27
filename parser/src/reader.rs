use std::num::NonZeroUsize;

use crate::Status;

/// A generic interface for reading from a data source.
pub trait Reader {
    /// Reads data from the source and advances the reader's position by the number
    /// of bytes read.
    ///
    /// Short reads are permitted, as is reading no data.
    ///
    /// # Arguments
    /// * `num_to_read` - The number of bytes that should be read.
    /// * `buffer` - The buffer to store the read bytes. Must be large enough to
    /// store at least `num_to_read` bytes. Must not be null.
    ///
    /// Returns `Status::General(GeneralStatus::OkCompleted)` if `num_to_read` bytes
    /// were read. `Status::General(GeneralStatus::OkPartial(u64))` if the number of
    /// bytes read is > 0 and < `num_to_read`, containing number of bytes read.
    /// If no bytes are read, then some other status must be returned.
    fn read(&mut self, num_to_read: NonZeroUsize, buffer: &mut [u8]) -> Status;

    /// Skips data from the source and advances the reader's position by the number
    /// of bytes skipped.
    ///
    /// Short skips are permitted, as is skipping no data. This is similar to the
    /// `read()` method, but does not store data in an output buffer.
    ///
    /// # Arguments
    /// * `num_to_skip` - The number of bytes that should be skipped.
    ///
    /// Returns `Status::General(GeneralStatus::OkCompleted)` if `num_to_skip` bytes
    /// were read. `Status::General(GeneralStatus::OkPartial(u64))` if the number of
    /// bytes skipped is > 0 and < `num_to_read`, containing number of bytes read.
    /// If no bytes are skipped, then some other status must be returned.
    fn skip(&mut self, num_to_skip: NonZeroUsize) -> Status;

    /// Gets the Reader's current absolute byte position in the stream.
    ///
    /// Implementations don't necessarily need to start from 0 (which might be the
    /// case if parsing is starting in the middle of a data source).
    fn position(&self) -> u64;
}
