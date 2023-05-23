#![doc = include_str!("../README.md")]

use std::{fs::File, io::Read, path::Path};

use anyhow::bail;
use mkvparser::{
    elements::Id, find_valid_element, parse_body, parse_header, Body, Element,
    Header,
};

fn insert_position(
    element: &mut Element,
    current_header: &Option<Header>,
    position: &mut Option<usize>,
) {
    element.header.position = *position;
    *position = position.map(|p| {
        let v = if let Body::Master = element.body {
            p + element.header.header_size
        } else {
            // It's safe to unwrap because all non-Master elements have a set size
            p + element.header.size.unwrap()
        };
        v + current_header.as_ref().map_or(0, |h| h.header_size)
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

    let mut current_header = None;

    loop {
        let num_read = file.read(&mut buffer[filled..])?;
        if num_read == 0 {
            if filled == 0 {
                // we have nothing left to read or parse
                break;
            } else if filled == buffer_size {
                // We might arrive at this condition if the next element does not fit
                // in the buffer, so we continuously get NeedData.
                // TODO(#59): we shouldn't need to get into this normally we only parse
                // headers.
                bail!("failed to parse the given file with buffer size of {buffer_size} bytes");
            }
        }

        let mut parse_buffer = &buffer[..(filled + num_read)];

        loop {
            if current_header.is_none() {
                match parse_header(parse_buffer) {
                    Ok((new_parse_buffer, header)) => {
                        current_header = Some(header);
                        parse_buffer = new_parse_buffer;
                    }
                    Err(mkvparser::Error::NeedData) => break,
                    Err(_) => {
                        // Attempt to find a valid element only if we have the full buffer
                        // and are reading it from the start. If we can't find a valid element
                        // in this case, then we'll bail.
                        if parse_buffer.len() == filled + num_read {
                            let (new_parse_buffer, corrupt_element) =
                                find_valid_element(parse_buffer)?;
                            push_corrupt_element(&mut elements, None, corrupt_element);
                            parse_buffer = new_parse_buffer;
                        } else {
                            break;
                        }
                    }
                }
            } else {
                match parse_body(parse_buffer, current_header.as_ref().unwrap()) {
                    Ok((new_parse_buffer, body)) => {
                        let mut element = Element {
                            header: current_header.take().unwrap(),
                            body,
                        };
                        insert_position(&mut element, &None, &mut position);
                        elements.push(element);
                        parse_buffer = new_parse_buffer;
                    }
                    Err(mkvparser::Error::NeedData) => break,
                    Err(_) => {
                        // Attempt to find a valid element only if we have the full buffer
                        // and are reading it from the start. If we can't find a valid element
                        // in this case, then we'll bail.
                        if parse_buffer.len() == filled + num_read {
                            let (new_parse_buffer, mut corrupt_element) =
                                find_valid_element(parse_buffer)?;
                            insert_position(&mut corrupt_element, &current_header, &mut position);
                            push_corrupt_element(
                                &mut elements,
                                current_header.take(),
                                corrupt_element,
                            );
                            parse_buffer = new_parse_buffer;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        filled = parse_buffer.len();
        let parse_buffer = Vec::from(parse_buffer);
        buffer[..filled].copy_from_slice(&parse_buffer);
    }
    Ok(elements)
}

fn push_corrupt_element(
    elements: &mut Vec<Element>,
    current_header: Option<Header>,
    mut corrupt_element: Element,
) {
    match elements.last_mut() {
        Some(last_element) if last_element.header.id == Id::corrupted() => {
            last_element.header = Header::new(
                Id::corrupted(),
                last_element.header.header_size + corrupt_element.header.header_size,
                last_element.header.body_size.unwrap()
                    + corrupt_element.header.body_size.unwrap()
                    + current_header.map_or(0, |c| c.header_size),
            );
        }
        _ => {
            *corrupt_element.header.size.as_mut().unwrap() +=
                current_header.map_or(0, |c| c.header_size);
            elements.push(corrupt_element)
        }
    }
}

#[cfg(test)]
mod tests {
    use mkvparser::Binary;

    use super::*;

    #[test]
    fn sequential_corrupt_elements() {
        let mut elements = vec![];
        let example_element = Element {
            header: Header {
                id: Id::corrupted(),
                header_size: 0,
                body_size: Some(4),
                size: Some(4),
                position: None,
            },
            body: Body::Binary(Binary::Corrupted),
        };
        push_corrupt_element(&mut elements, None, example_element.clone());
        push_corrupt_element(&mut elements, None, example_element);

        assert_eq!(elements.len(), 1);
        assert_eq!(
            elements[0],
            Element {
                header: Header {
                    id: Id::corrupted(),
                    header_size: 0,
                    body_size: Some(8),
                    size: Some(8),
                    position: None,
                },
                body: Body::Binary(Binary::Corrupted),
            }
        )
    }
}
