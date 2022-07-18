use crate::status::{GeneralStatus, Status};
use crate::Reader;

/// A simple reader that reads data from a buffer of bytes.
pub struct BufferReader {
    // Stores the byte buffer from which data is read.
    data: Vec<u8>,
    // The position of the reader in the byte buffer.
    pos: usize,
}

impl BufferReader {
    /// Creates a new `BufferReader` populated with the provided data.
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, pos: 0 }
    }

    /// Gets the total size of the buffer.
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl Reader for BufferReader {
    fn read(&mut self, num_to_read: usize, buffer: &mut [u8]) -> Status {
        assert!(num_to_read > 0);

        let expected = num_to_read;
        let mut num_actually_read = num_to_read;

        let num_remaining = self.data.len() - self.pos;
        if num_remaining == 0 {
            return GeneralStatus::EndOfFile.into();
        }

        if num_actually_read > num_remaining {
            num_actually_read = num_remaining;
        }

        buffer[..(num_actually_read as usize)]
            .copy_from_slice(&self.data[self.pos..(self.pos + num_actually_read)]);

        self.pos += num_actually_read;

        if num_actually_read != expected {
            return GeneralStatus::OkPartial(num_actually_read as u64).into();
        }
        GeneralStatus::OkCompleted.into()
    }

    fn skip(&mut self, num_to_skip: u64) -> Status {
        assert!(num_to_skip > 0);

        let expected = num_to_skip;
        let mut num_actually_skipped = num_to_skip;

        let num_remaining = (self.data.len() - self.pos) as u64;
        if num_remaining == 0 {
            return GeneralStatus::EndOfFile.into();
        }

        if num_actually_skipped > num_remaining {
            num_actually_skipped = num_remaining;
        }

        self.pos += num_actually_skipped as usize;

        if num_actually_skipped != expected {
            return GeneralStatus::OkPartial(num_actually_skipped).into();
        }

        GeneralStatus::OkCompleted.into()
    }

    fn position(&self) -> u64 {
        self.pos as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment() {
        let mut buffer = [0u8; 4];
        let mut reader = BufferReader::new(vec![]);
        assert_eq!(reader.size(), 0);

        let mut status = reader.read(buffer.len(), &mut buffer);
        assert_eq!(status, GeneralStatus::EndOfFile);

        reader = BufferReader::new(vec![1, 2, 3, 4]);
        assert_eq!(reader.size(), 4);

        status = reader.read(2, &mut buffer);
        assert_eq!(status, GeneralStatus::OkCompleted);

        reader = BufferReader::new(vec![5, 6, 7, 8]);
        status = reader.read(2, &mut buffer[2..]);
        assert_eq!(status, GeneralStatus::OkCompleted);

        let expected = [1, 2, 5, 6];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn empty() {
        let mut buffer = [0u8; 1];
        let mut reader = BufferReader::new(vec![]);

        let mut status = reader.read(buffer.len(), &mut buffer);
        assert_eq!(status, GeneralStatus::EndOfFile);

        status = reader.skip(1);
        assert_eq!(status, GeneralStatus::EndOfFile);
    }

    #[test]
    fn read() {
        let mut buffer = [0u8; 15];
        let mut reader = BufferReader::new(Vec::from_iter(0..=9));

        let mut status = reader.read(5, &mut buffer);
        assert_eq!(status, GeneralStatus::OkCompleted);

        status = reader.read(10, &mut buffer[5..]);
        assert_eq!(status, GeneralStatus::OkPartial(5));

        let expected = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0];
        assert_eq!(buffer, expected);

        status = reader.read(buffer.len(), &mut buffer);
        assert_eq!(status, GeneralStatus::EndOfFile);
    }

    #[test]
    fn skip() {
        let mut reader = BufferReader::new(Vec::from_iter(0..=9));

        let mut status = reader.skip(3);
        assert_eq!(status, GeneralStatus::OkCompleted);

        status = reader.skip(10);
        assert_eq!(status, GeneralStatus::OkPartial(7));

        status = reader.skip(1);
        assert_eq!(status, GeneralStatus::EndOfFile);
    }

    #[test]
    fn read_and_skip() {
        let mut buffer = [0u8; 10];
        let mut reader = BufferReader::new(Vec::from_iter((0..=9).rev()));

        let mut status = reader.read(5, &mut buffer);
        assert_eq!(status, GeneralStatus::OkCompleted);

        status = reader.skip(3);
        assert_eq!(status, GeneralStatus::OkCompleted);

        status = reader.read(5, &mut buffer[5..]);
        assert_eq!(status, GeneralStatus::OkPartial(2));

        let expected = [9, 8, 7, 6, 5, 1, 0, 0, 0, 0];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn position() {
        let mut buffer = [0u8; 10];
        let mut reader = BufferReader::new(Vec::from_iter((0..=9).rev()));

        let mut status = reader.read(5, &mut buffer);
        assert_eq!(status, GeneralStatus::OkCompleted);
        assert_eq!(reader.position(), 5);

        status = reader.skip(3);
        assert_eq!(status, GeneralStatus::OkCompleted);
        assert_eq!(reader.position(), 8);

        status = reader.read(5, &mut buffer[5..]);
        assert_eq!(status, GeneralStatus::OkPartial(2));
        assert_eq!(reader.position(), 10);

        let expected = [9, 8, 7, 6, 5, 1, 0, 0, 0, 0];
        assert_eq!(buffer, expected);
    }
}
