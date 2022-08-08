use std::ops::Not;
use std::{
    fs::File,
    io::{self, Read},
};

use clap::Parser;
use nom::{
    bytes::streaming::take,
    combinator::peek,
    error::{Error, ErrorKind},
    Err, IResult,
};
use serde::Serialize;

mod ebml;
mod elements;

use crate::elements::*;

fn parse_id(input: &[u8]) -> IResult<&[u8], Id> {
    let (input, first_byte) = peek(take(1usize))(input)?;
    let first_byte = first_byte[0];

    let num_bytes = count_leading_zero_bits(first_byte) + 1;

    // IDs can only have up to 4 bytes
    if num_bytes > 4 {
        return Err(Err::Failure(Error::new(input, ErrorKind::Fail)));
    }

    let (input, varint_bytes) = take(num_bytes)(input)?;
    // any efficient way to avoid this copy here?
    let mut value_buffer = [0u8; 4];
    value_buffer[(4 - varint_bytes.len())..].copy_from_slice(varint_bytes);
    let id = u32::from_be_bytes(value_buffer);

    Ok((input, Id::new(id)))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct Header {
    pub id: Id,
    pub header_size: usize,
    #[serde(skip_serializing)]
    pub body_size: u64,
    pub size: u64,
}

impl Header {
    fn new(id: Id, header_size: usize, body_size: u64) -> Header {
        Header {
            id,
            header_size,
            body_size,
            size: header_size as u64 + body_size,
        }
    }
}

// TODO: turn into a loop
fn count_leading_zero_bits(input: u8) -> u8 {
    if input & 0b10000000 != 0 {
        0
    } else if input == 0 {
        8
    } else {
        count_leading_zero_bits(input << 1) + 1
    }
}

fn parse_varint(first_input: &[u8]) -> IResult<&[u8], u64> {
    let (input, first_byte) = peek(take(1usize))(first_input)?;
    let first_byte = first_byte[0];

    let vint_prefix_size = count_leading_zero_bits(first_byte) + 1;

    // Maximum 8 bytes, i.e. first byte can't be 0
    if vint_prefix_size > 8 {
        return Err(Err::Failure(Error::new(first_input, ErrorKind::Fail)));
    }

    let (input, varint_bytes) = take(vint_prefix_size)(input)?;
    // any efficient way to avoid this copy here?
    let mut value_buffer = [0u8; 8];
    value_buffer[(8 - varint_bytes.len())..].copy_from_slice(varint_bytes);
    let mut value = u64::from_be_bytes(value_buffer);

    // discard varint prefix (zeros + market bit)
    let num_bits_in_value = 7 * vint_prefix_size;
    value &= (1 << num_bits_in_value) - 1;

    Ok((input, value))
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let initial_len = input.len();
    let (input, id) = parse_id(input)?;
    let (input, body_size) = parse_varint(input)?;

    let header_size = initial_len - input.len();

    Ok((input, Header::new(id, header_size, body_size)))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
enum Lacing {
    Xiph,
    Ebml,
    FixedSize,
}

// https://www.matroska.org/technical/basics.html#block-structure
#[derive(Debug, Clone, PartialEq, Serialize)]
struct Block {
    track_number: u64,
    timestamp: i16,
    #[serde(skip_serializing_if = "Not::not")]
    invisible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    lacing: Option<Lacing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_frames: Option<u8>,
}

// https://www.matroska.org/technical/basics.html#simpleblock-structure
#[derive(Debug, Clone, PartialEq, Serialize)]
struct SimpleBlock {
    track_number: u64,
    timestamp: i16,
    #[serde(skip_serializing_if = "Not::not")]
    keyframe: bool,
    #[serde(skip_serializing_if = "Not::not")]
    invisible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    lacing: Option<Lacing>,
    #[serde(skip_serializing_if = "Not::not")]
    discardable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_frames: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum BinaryValue {
    Hidden,
    SeekId(Id),
    SimpleBlock(SimpleBlock),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum Body {
    Master,
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Utf8(String),
    Date(i64),
    Binary(BinaryValue),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct Element {
    #[serde(flatten)]
    pub header: Header,
    #[serde(rename = "value")]
    pub body: Body,
}

fn parse_element(input: &[u8]) -> IResult<&[u8], Element> {
    let (input, header) = parse_header(input)?;
    let element_type = header.id.get_type();

    let (input, body) = match element_type {
        Type::Master => Ok((input, Body::Master)),
        Type::Unsigned => {
            parse_int(&header, input).map(|(input, value)| (input, Body::Unsigned(value)))
        }
        Type::Signed => {
            parse_int(&header, input).map(|(input, value)| (input, Body::Signed(value)))
        }
        Type::Float => {
            parse_float(&header, input).map(|(input, value)| (input, Body::Float(value)))
        }
        Type::String => {
            parse_string(&header, input).map(|(input, value)| (input, Body::String(value)))
        }
        Type::Utf8 => parse_string(&header, input).map(|(input, value)| (input, Body::Utf8(value))),
        Type::Date => todo!(),
        Type::Binary => parse_binary(&header, input).map(|(input, value)| {
            let binary_value = match header.id {
                Id::SeekId => BinaryValue::SeekId(parse_id(&value).unwrap().1),
                Id::SimpleBlock => BinaryValue::SimpleBlock(parse_simple_block(&value).unwrap().1),
                Id::Block => BinaryValue::Block(parse_block(&value).unwrap().1),
                _ => BinaryValue::Hidden,
            };
            (input, Body::Binary(binary_value))
        }),
    }?;
    let element = Element { header, body };
    Ok((input, element))
}

fn parse_string<'a>(metadata: &Header, input: &'a [u8]) -> IResult<&'a [u8], String> {
    let (input, string_bytes) = take(metadata.body_size)(input)?;
    let value = String::from_utf8(string_bytes.to_vec())
        .map_err(|_| Err::Failure(Error::new(input, ErrorKind::Fail)))?;

    Ok((input, value))
}

fn parse_binary<'a>(metadata: &Header, input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
    let (input, bytes) = take(metadata.body_size)(input)?;
    let value = Vec::from(bytes);

    Ok((input, value))
}

trait Integer64FromBigEndianBytes {
    fn from_be_bytes(input: [u8; 8]) -> Self;
}

impl Integer64FromBigEndianBytes for u64 {
    fn from_be_bytes(input: [u8; 8]) -> Self {
        u64::from_be_bytes(input)
    }
}

impl Integer64FromBigEndianBytes for i64 {
    fn from_be_bytes(input: [u8; 8]) -> Self {
        i64::from_be_bytes(input)
    }
}

fn parse_int<'a, T: Integer64FromBigEndianBytes>(
    metadata: &Header,
    input: &'a [u8],
) -> IResult<&'a [u8], T> {
    let (input, int_bytes) = take(metadata.body_size)(input)?;
    // FIXME: any efficient way to avoid this copy here?
    let mut value_buffer = [0u8; 8];
    value_buffer[(8 - int_bytes.len())..].copy_from_slice(int_bytes);
    let value = T::from_be_bytes(value_buffer);

    Ok((input, value))
}

fn parse_float<'a>(metadata: &Header, input: &'a [u8]) -> IResult<&'a [u8], f64> {
    let (input, float_bytes) = take(metadata.body_size)(input)?;

    if metadata.body_size == 4 {
        let value = f32::from_be_bytes(float_bytes.try_into().unwrap()) as f64;
        Ok((input, value))
    } else if metadata.body_size == 8 {
        let value = f64::from_be_bytes(float_bytes.try_into().unwrap());
        Ok((input, value))
    } else {
        todo!()
    }
}

fn parse_i16(input: &[u8]) -> IResult<&[u8], i16> {
    let (input, bytes) = take(2usize)(input)?;
    let value = i16::from_be_bytes(bytes.try_into().unwrap());
    Ok((input, value))
}

fn is_invisible(flags: u8) -> bool {
    (flags & (1 << 3)) != 0
}

fn get_lacing(flags: u8) -> Option<Lacing> {
    match (flags & (0b110)) >> 1 {
        0b00 => None,
        0b01 => Some(Lacing::Xiph),
        0b11 => Some(Lacing::Ebml),
        0b10 => Some(Lacing::FixedSize),
        _ => unreachable!(),
    }
}

fn parse_block(input: &[u8]) -> IResult<&[u8], Block> {
    let (input, track_number) = parse_varint(input)?;
    let (input, timestamp) = parse_i16(input)?;
    let (input, flags) = take(1usize)(input)?;
    let flags = flags[0];

    let invisible = is_invisible(flags);
    let lacing = get_lacing(flags);
    let (input, num_frames) = if lacing != None {
        let (input, next_byte) = take(1usize)(input)?;
        let num_frames = next_byte[0];
        (input, Some(num_frames + 1))
    } else {
        (input, None)
    };

    Ok((
        input,
        Block {
            track_number,
            timestamp,
            invisible,
            lacing,
            num_frames,
        },
    ))
}

fn parse_simple_block(input: &[u8]) -> IResult<&[u8], SimpleBlock> {
    let (input, track_number) = parse_varint(input)?;
    let (input, timestamp) = parse_i16(input)?;
    let (input, flags) = take(1usize)(input)?;
    let flags = flags[0];

    let keyframe = (flags & (1 << 7)) != 0;
    let invisible = is_invisible(flags);
    let lacing = get_lacing(flags);
    let discardable = (flags & 0b1) != 0;
    let (input, num_frames) = if lacing != None {
        let (input, next_byte) = take(1usize)(input)?;
        let num_frames = next_byte[0];
        (input, Some(num_frames + 1))
    } else {
        (input, None)
    };

    Ok((
        input,
        SimpleBlock {
            track_number,
            timestamp,
            keyframe,
            invisible,
            lacing,
            discardable,
            num_frames,
        },
    ))
}

#[derive(Debug, PartialEq, Serialize)]
struct MasterElement {
    #[serde(flatten)]
    header: Header,
    children: Vec<ElementTree>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
enum ElementTree {
    Normal(Element),
    Master(MasterElement),
}

fn build_element_trees(elements: &[Element]) -> Vec<ElementTree> {
    let mut trees = Vec::<ElementTree>::new();

    let mut index = 0;
    while index < elements.len() {
        let element = &elements[index];
        match element.body {
            Body::Master => {
                let mut size_remaining = element.header.body_size;
                // TODO: no need to copy here
                let mut children = Vec::<Element>::new();
                while size_remaining > 0 {
                    index += 1;
                    let next_child = &elements[index];
                    size_remaining -= if let Body::Master = next_child.body {
                        // Master elements' body size should not count in the recursion
                        // as the children would duplicate the size count, so
                        // we only consider the header size on the calculation.
                        next_child.header.header_size as u64
                    } else {
                        next_child.header.size
                    };
                    children.push(next_child.clone());
                }
                trees.push(ElementTree::Master(MasterElement {
                    header: element.header.clone(),
                    children: build_element_trees(&children),
                }));
            }
            _ => {
                trees.push(ElementTree::Normal(element.clone()));
            }
        }
        index += 1;
    }
    trees
}

fn print_element_trees(elements: &[Element], format: &str) {
    let element_trees = build_element_trees(elements);
    let serialized = if format == "json" {
        serde_json::to_string_pretty(&element_trees).unwrap()
    } else {
        serde_yaml::to_string(&element_trees).unwrap()
    };
    println!("{}", serialized);
}

#[cfg(test)]
mod tests {
    use nom::Needed;

    use super::*;

    const EMPTY: &[u8] = &[];

    #[test]
    fn test_count_leading_zero_bits() {
        assert_eq!(count_leading_zero_bits(0b10000000), 0);
        assert_eq!(count_leading_zero_bits(0b01000000), 1);
        assert_eq!(count_leading_zero_bits(0b00000001), 7);
        assert_eq!(count_leading_zero_bits(0b00000000), 8);
    }

    #[test]
    fn test_parse_id() {
        assert_eq!(parse_id(&[0x1A, 0x45, 0xDF, 0xA3]), Ok((EMPTY, Id::Ebml)));
        assert_eq!(parse_id(&[0x42, 0x86]), Ok((EMPTY, Id::EbmlVersion)));
        assert_eq!(parse_id(&[0x23, 0x83, 0xE3]), Ok((EMPTY, Id::FrameRate)));

        // 1 byte missing from FrameRate (3-bytes long)
        assert_eq!(
            parse_id(&[0x23, 0x83]),
            Err(Err::Incomplete(Needed::Size(1.try_into().unwrap())))
        );

        // Longer than 4 bytes
        const FAILURE_INPUT: &[u8] = &[0x08, 0x45, 0xDF, 0xA3];
        assert_eq!(
            parse_id(FAILURE_INPUT),
            Err(Err::Failure(Error::new(FAILURE_INPUT, ErrorKind::Fail)))
        );
    }

    #[test]
    fn test_parse_varint() {
        assert_eq!(parse_varint(&[0x9F]), Ok((EMPTY, 31)));
        assert_eq!(parse_varint(&[0x81]), Ok((EMPTY, 1)));
        assert_eq!(parse_varint(&[0x53, 0xAC]), Ok((EMPTY, 5036)));

        const INVALID_VARINT: &[u8] = &[0x00, 0xAC];
        assert_eq!(
            parse_varint(INVALID_VARINT),
            Err(Err::Failure(Error::new(INVALID_VARINT, ErrorKind::Fail)))
        );
    }

    #[test]
    fn test_parse_element_metadata() {
        const INPUT: &[u8] = &[0x1A, 0x45, 0xDF, 0xA3, 0x9F];
        assert_eq!(
            parse_header(INPUT),
            Ok((EMPTY, Header::new(Id::Ebml, 5, 31)))
        );
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse_string(&Header::new(Id::DocType, 3, 4), &[0x77, 0x65, 0x62, 0x6D]),
            Ok((EMPTY, "webm".to_string()))
        )
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(
            parse_int(&Header::new(Id::EbmlVersion, 3, 1), &[0x01]),
            Ok((EMPTY, 1u64))
        )
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(
            parse_float(&Header::new(Id::Duration, 3, 4), &[0x45, 0x7A, 0x30, 0x00]),
            Ok((EMPTY, 4003.))
        )
    }

    #[test]
    fn test_parse_binary() {
        assert_eq!(
            parse_binary(&Header::new(Id::SeekId, 3, 4), &[0x15, 0x49, 0xA9, 0x66]),
            Ok((EMPTY, vec![0x15, 0x49, 0xA9, 0x66]))
        )
    }

    #[test]
    fn test_parse_master_element() {
        const INPUT: &[u8] = &[
            0x1A, 0x45, 0xDF, 0xA3, 0x9F, 0x42, 0x86, 0x81, 0x01, 0x42, 0xF7, 0x81, 0x01, 0x42,
            0xF2, 0x81, 0x04, 0x42, 0xF3, 0x81, 0x08, 0x42, 0x82, 0x84, 0x77, 0x65, 0x62, 0x6D,
            0x42, 0x87, 0x81, 0x04, 0x42, 0x85, 0x81, 0x02,
        ];

        let result = parse_element(INPUT);
        assert_eq!(
            result,
            Ok((
                &INPUT[5..],
                Element {
                    header: Header::new(Id::Ebml, 5, 31),
                    body: Body::Master
                }
            ))
        );

        println!("{}", serde_yaml::to_string(&(result.unwrap().1)).unwrap());
    }

    #[test]
    fn test_parse_seek_id() {
        assert_eq!(
            parse_element(&[0x53, 0xAB, 0x84, 0x15, 0x49, 0xA9, 0x66]),
            Ok((
                EMPTY,
                Element {
                    header: Header::new(Id::SeekId, 3, 4),
                    body: Body::Binary(BinaryValue::SeekId(Id::Info))
                }
            ))
        );
    }

    #[test]
    fn test_parse_crc32() {
        assert_eq!(
            parse_element(&[0xBF, 0x84, 0xAF, 0x93, 0x97, 0x18]),
            Ok((
                EMPTY,
                Element {
                    header: Header::new(Id::Crc32, 2, 4),
                    body: Body::Binary(BinaryValue::Hidden)
                }
            ))
        );
    }

    #[test]
    fn test_parse_empty() {
        assert_eq!(
            parse_element(&[0x63, 0xC0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            Ok((
                EMPTY,
                Element {
                    header: Header::new(Id::Targets, 10, 0),
                    body: Body::Master
                }
            ))
        );
    }

    #[test]
    fn test_parse_block() {
        assert_eq!(
            parse_block(&[0x81, 0x0F, 0x7A, 0x00]),
            Ok((
                EMPTY,
                Block {
                    track_number: 1,
                    timestamp: 3962,
                    invisible: false,
                    lacing: None,
                    num_frames: None
                }
            ))
        )
    }

    #[test]
    fn test_parse_simple_block() {
        assert_eq!(
            parse_simple_block(&[0x81, 0x00, 0x53, 0x00]),
            Ok((
                EMPTY,
                SimpleBlock {
                    track_number: 1,
                    timestamp: 83,
                    keyframe: false,
                    invisible: false,
                    lacing: None,
                    discardable: false,
                    num_frames: None,
                }
            ))
        )
    }

    #[test]
    fn test_build_element_trees() {
        let elements = [
            Element {
                header: Header::new(Id::Ebml, 5, 31),
                body: Body::Master,
            },
            Element {
                header: Header::new(Id::EbmlVersion, 3, 1),
                body: Body::Unsigned(1),
            },
            Element {
                header: Header::new(Id::EbmlReadVersion, 3, 1),
                body: Body::Unsigned(1),
            },
            Element {
                header: Header::new(Id::EbmlMaxIdLength, 3, 1),
                body: Body::Unsigned(4),
            },
            Element {
                header: Header::new(Id::EbmlMaxSizeLength, 3, 1),
                body: Body::Unsigned(8),
            },
            Element {
                header: Header::new(Id::DocType, 3, 4),
                body: Body::String("webm".to_string()),
            },
            Element {
                header: Header::new(Id::DocTypeVersion, 3, 1),
                body: Body::Unsigned(4),
            },
            Element {
                header: Header::new(Id::DocTypeReadVersion, 3, 1),
                body: Body::Unsigned(2),
            },
        ];

        let expected = vec![ElementTree::Master(MasterElement {
            header: Header::new(Id::Ebml, 5, 31),
            children: vec![
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlVersion, 3, 1),
                    body: Body::Unsigned(1),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlReadVersion, 3, 1),
                    body: Body::Unsigned(1),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlMaxIdLength, 3, 1),
                    body: Body::Unsigned(4),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlMaxSizeLength, 3, 1),
                    body: Body::Unsigned(8),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::DocType, 3, 4),
                    body: Body::String("webm".to_string()),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::DocTypeVersion, 3, 1),
                    body: Body::Unsigned(4),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::DocTypeReadVersion, 3, 1),
                    body: Body::Unsigned(2),
                }),
            ],
        })];

        assert_eq!(build_element_trees(&elements), expected);
    }
}

/// WebM dump
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to be dumped
    filename: String,

    /// Output in JSON format, rather than the default YAML
    #[clap(short, long, default_value = "yaml")]
    format: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.filename)?;

    // TODO: read chunked to not load entire video in memory.
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;

    let mut elements = Vec::<Element>::new();

    let mut read_buffer = &buffer[..];
    loop {
        match parse_element(read_buffer) {
            Ok((new_read_buffer, element)) => {
                elements.push(element);
                if new_read_buffer.is_empty() {
                    break;
                }
                read_buffer = new_read_buffer;
            }
            Err(nom::Err::Incomplete(needed)) => {
                println!(
                    "Needed: {:?}\nPartial result:\n{}",
                    needed,
                    serde_yaml::to_string(&elements).unwrap()
                );
                todo!("Partial reads not implemented")
            }
            Err(_) => {
                panic!("Something is wrong");
            }
        }
    }

    print_element_trees(&elements, &args.format);

    Ok(())
}
