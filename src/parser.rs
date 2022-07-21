use crate::{Callback, Status};

pub trait Parser {
    fn feed(&mut self, callback: &mut dyn Callback) -> Status;
}
