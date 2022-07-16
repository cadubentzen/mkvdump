use crate::status::Status;

pub trait Reader {
    fn read(&mut self, num_to_read: usize, buffer: &mut [u8]) -> Status;
    fn skip(&mut self, num_to_skip: u64) -> Status;
    fn position(&self) -> u64;
}
