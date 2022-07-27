use webm_parser::{BufferReader, Reader};

fn main() {
    let mut buffer = [0u8; 10];
    let mut reader = BufferReader::new(Vec::from_iter((0..=9).rev()));

    let status = reader.read(5.try_into().unwrap(), &mut buffer);
    println!("Status: {:?}", status);
}
