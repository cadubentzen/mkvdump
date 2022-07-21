use crate::{Callback, Status};

trait Parser {
    fn feed(&mut self, callback: &mut dyn Callback) -> Status;
}
