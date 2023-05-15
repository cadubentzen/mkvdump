use std::{fs::File, io::Read, path::Path};

use anyhow::bail;
use mkvparser::{find_valid_element, parse_element, Body, Element};

fn insert_position(element: &mut Element, position: &mut Option<usize>) {
    element.header.position = *position;
    *position = position.map(|p| {
        if let Body::Master = element.body {
            p + element.header.header_size
        } else {
            // It's safe to unwrap because all non-Master elements have a set size
            p + element.header.size.unwrap()
        }
    });
}

#[doc(hidden)]
pub fn parse_elements_from_file(
    path: impl AsRef<Path>,
    show_positions: bool,
    buffer_size: u64,
) -> anyhow::Result<Vec<Element>> {
    let mut file = File::open(path)?;
    let file_length = file.metadata()?.len();

    let buffer_size = file_length.min(buffer_size).try_into().unwrap();
    let mut buffer = vec![0; buffer_size];
    let mut filled = 0;
    let mut elements = Vec::<Element>::new();
    let mut position = show_positions.then_some(0);

    loop {
        let num_read = file.read(&mut buffer[filled..])?;
        if num_read == 0 {
            if filled == 0 {
                // we have nothing left to read or parse
                break;
            } else if filled == buffer_size {
                // We might arrive at this condition for two reasons:
                // - next element does not fit in the buffer, so we continuously get NeedData
                // - buffer is completely filled with corrupt data
                bail!("failed to parse the given file with buffer size of {buffer_size} bytes");
            }
        }

        let mut parse_buffer = &buffer[..(filled + num_read)];
        loop {
            match parse_element(&parse_buffer) {
                Ok((new_parse_buffer, mut element)) => {
                    println!("{element:?}");
                    insert_position(&mut element, &mut position);
                    elements.push(element);
                    parse_buffer = new_parse_buffer;
                }
                Err(e) => {
                    match e {
                        mkvparser::Error::NeedData => break,
                        _ => {
                            match find_valid_element(parse_buffer) {
                                Ok((new_parse_buffer, mut corrupt_element)) => {
                                    // TODO: merge consecutive corrupt elements
                                    insert_position(&mut corrupt_element, &mut position);
                                    elements.push(corrupt_element);
                                    parse_buffer = new_parse_buffer;
                                }
                                Err(_) => {
                                    // TODO: if we can't find a valid element, we could consume the
                                    // whole buffer as a corrupt element. That way we wouldn't bail if
                                    // the corrupt region is bigger than the buffer size.
                                    break;
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        filled = parse_buffer.len();
        let parse_buffer = Vec::from(parse_buffer);
        buffer[..filled].copy_from_slice(&parse_buffer);
    }
    Ok(elements)
}
