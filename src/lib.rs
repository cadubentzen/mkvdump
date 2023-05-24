#![doc = include_str!("../README.md")]

use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use anyhow::bail;
use mkvparser::{
    elements::{Id, Type},
    parse_body, parse_corrupt, parse_header, peek_binary, Body, Element, Error, Header,
};

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

type IResult<T, O> = mkvparser::Result<(T, O)>;

struct ShortParsed {
    element: Element,
    bytes_to_be_skipped: usize,
}

fn parse_short(input: &[u8]) -> IResult<&[u8], ShortParsed> {
    let (input, header) = parse_header(input)?;
    if header.id.get_type() != Type::Binary {
        let (input, body) = parse_body(&header, input)?;
        Ok((
            input,
            ShortParsed {
                element: Element { header, body },
                bytes_to_be_skipped: 0,
            },
        ))
    } else {
        let (input, binary) = peek_binary(&header, input)?;
        let body_size = header.body_size.ok_or(Error::ForbiddenUnknownSize)?;
        Ok((
            input,
            ShortParsed {
                element: Element {
                    header,
                    body: Body::Binary(binary),
                },
                bytes_to_be_skipped: body_size,
            },
        ))
    }
}

fn parse_short_corrupt<'a>(input: &'a [u8], is_corrupt: &mut bool) -> (&'a [u8], ShortParsed) {
    let (input, corrupt_element) = parse_corrupt(input);
    if !input.is_empty() {
        *is_corrupt = false;
    }
    (
        input,
        ShortParsed {
            element: corrupt_element,
            bytes_to_be_skipped: 0,
        },
    )
}

fn parse_short_or_corrupt<'a>(
    input: &'a [u8],
    is_corrupt: &mut bool,
) -> IResult<&'a [u8], ShortParsed> {
    let parsed_short = if *is_corrupt {
        if input.is_empty() {
            Err(Error::NeedData)
        } else {
            Ok(parse_short_corrupt(input, is_corrupt))
        }
    } else {
        parse_short(input)
    };

    match parsed_short {
        Ok((input, short_parsed)) => Ok((input, short_parsed)),
        Err(Error::NeedData) => Err(Error::NeedData),
        Err(_) => {
            *is_corrupt = true;
            Ok(parse_short_corrupt(input, is_corrupt))
        }
    }
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
    let mut is_corrupt = false;

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
            match parse_short_or_corrupt(parse_buffer, &mut is_corrupt) {
                Ok((
                    new_parse_buffer,
                    ShortParsed {
                        mut element,
                        bytes_to_be_skipped,
                    },
                )) => {
                    insert_position(&mut element, &mut position);

                    if element.header.id == Id::corrupted() {
                        push_corrupt_element(&mut elements, element);
                    } else {
                        elements.push(element);
                    }

                    if new_parse_buffer.len() >= bytes_to_be_skipped {
                        parse_buffer = &new_parse_buffer[bytes_to_be_skipped..];
                    } else {
                        file.seek(std::io::SeekFrom::Current(
                            (bytes_to_be_skipped - new_parse_buffer.len()) as i64,
                        ))?;
                        parse_buffer = &[];
                    }
                }
                Err(_) => break,
            }
        }
        filled = parse_buffer.len();
        let parse_buffer = Vec::from(parse_buffer);
        buffer[..filled].copy_from_slice(&parse_buffer);
    }
    Ok(elements)
}

fn push_corrupt_element(elements: &mut Vec<Element>, corrupt_element: Element) {
    match elements.last_mut() {
        Some(last_element) if last_element.header.id == Id::corrupted() => {
            last_element.header = Header::new(
                Id::corrupted(),
                last_element.header.header_size + corrupt_element.header.header_size,
                last_element.header.body_size.unwrap() + corrupt_element.header.body_size.unwrap(),
            );
        }
        _ => elements.push(corrupt_element),
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
        push_corrupt_element(&mut elements, example_element.clone());
        push_corrupt_element(&mut elements, example_element);

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
