#![doc = include_str!("../README.md")]

use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use mkvparser::{
    elements::{Id, Type},
    parse_body, parse_corrupt, parse_header, peek_binary, Binary, Body, Element, Error, Header,
};

const DEFAULT_BUFFER_SIZE: u64 = 8192;

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

// For all element types except Binary, we can just parse the body, consuming all
// bytes in it. Binary bodies can be rather large, but:
// - we are not going to display their full payload in the dump anyways
// - we don't want to load those large buffers in memory
// so we just peek the first bytes in the beginning for some binary sub-types,
// summarize the payload or serialize short ones.
// For the binary bodies, since we're only peeking the buffer and not consuming it,
// we return to the caller how many bytes should be skipped.
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

fn parse_short_corrupt<'a>(
    input: &'a [u8],
    is_corrupt: &mut bool,
) -> IResult<&'a [u8], ShortParsed> {
    let (input, corrupt_element) = parse_corrupt(input)?;
    // If we fully consume the buffer as a corrupt region, we are still in
    // a "corrupt state", so the caller should directly parse a
    // corrupt region again until some valid element is found instead of
    // attempting to parse an element (it could happen that parsing from
    // the wrong start byte yields valid elements and the parser never
    // returns to a valid state again).
    if !input.is_empty() {
        *is_corrupt = false;
    }
    Ok((
        input,
        ShortParsed {
            element: corrupt_element,
            bytes_to_be_skipped: 0,
        },
    ))
}

fn parse_short_or_corrupt<'a>(
    input: &'a [u8],
    is_corrupt: &mut bool,
) -> IResult<&'a [u8], ShortParsed> {
    let parsed_short = if *is_corrupt {
        parse_short_corrupt(input, is_corrupt)
    } else {
        parse_short(input)
    };

    match parsed_short {
        Ok((input, short_parsed)) => Ok((input, short_parsed)),
        Err(Error::NeedData) => Err(Error::NeedData),
        Err(_) => {
            *is_corrupt = true;
            parse_short_corrupt(input, is_corrupt)
        }
    }
}

#[doc(hidden)]
pub fn parse_elements_from_file(
    path: impl AsRef<Path>,
    show_positions: bool,
) -> anyhow::Result<Vec<Element>> {
    let mut file = File::open(path)?;
    let file_length = file.metadata()?.len();

    let buffer_size = file_length.min(DEFAULT_BUFFER_SIZE).try_into().unwrap();
    let mut buffer = vec![0; buffer_size];
    let mut filled = 0;
    let mut elements = Vec::<Element>::new();
    let mut position = show_positions.then_some(0);
    let mut is_corrupt = false;

    loop {
        let num_read = file.read(&mut buffer[filled..])?;
        let mut parse_buffer = &buffer[..(filled + num_read)];

        if num_read == 0 {
            // If some bytes are still to be parsed but nothing was read,
            // append a final corrupt element.
            if !parse_buffer.is_empty() {
                push_corrupt_element(
                    &mut elements,
                    Element {
                        header: Header::new(Id::corrupted(), 0, parse_buffer.len()),
                        body: Body::Binary(Binary::Corrupted),
                    },
                )
            }

            // we have nothing left to read or parse
            break;
        }

        while let Ok((
            new_parse_buffer,
            ShortParsed {
                mut element,
                bytes_to_be_skipped,
            },
        )) = parse_short_or_corrupt(parse_buffer, &mut is_corrupt)
        {
            insert_position(&mut element, &mut position);

            if element.header.id == Id::corrupted() {
                push_corrupt_element(&mut elements, element);
            } else {
                elements.push(element);
            }

            if new_parse_buffer.len() >= bytes_to_be_skipped {
                // If the binary body is already in our buffer, just skip in
                // the buffer
                parse_buffer = &new_parse_buffer[bytes_to_be_skipped..];
            } else {
                // Else, skip the remaining bytes in the buffer and seek in the file.
                file.seek(std::io::SeekFrom::Current(
                    (bytes_to_be_skipped - new_parse_buffer.len()) as i64,
                ))?;
                parse_buffer = &[];
            }
        }

        filled = parse_buffer.len();
        let parse_buffer = Vec::from(parse_buffer);
        buffer[..filled].copy_from_slice(&parse_buffer);
    }
    Ok(elements)
}

// While pushing corrupt elements, we check whether the last element was also corrupt
// to merge the corrupt area rather than appending a new element.
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
