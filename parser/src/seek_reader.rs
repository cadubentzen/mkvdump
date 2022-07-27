use std::io::{Read, Seek};

pub trait SeekRead: Seek + Read {}
impl<T: Seek + Read> SeekRead for T {}
