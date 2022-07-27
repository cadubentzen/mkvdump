use std::io::{Read, Seek};

use crate::{Callback, SeekRead, Status};

pub trait Parser {
    fn feed<R: Read + Seek>(
        &mut self,
        callback: &mut dyn Callback,
        reader: &mut dyn SeekRead,
    ) -> Status;
}
