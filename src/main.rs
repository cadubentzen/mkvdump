use std::ops::Not;
use std::{
    fs::File,
    io::{self, Read},
};

use chrono::prelude::*;
use clap::Parser;
use nom::{
    bytes::streaming::take,
    combinator::peek,
    error::{Error, ErrorKind},
    Err, IResult,
};
use serde::{Serialize, Serializer};

mod ebml;
mod elements;
mod enumerations;

use crate::elements::{Id, Type};
use crate::enumerations::Enumeration;

fn parse_id(input: &[u8]) -> IResult<&[u8], Id> {
    let (input, first_byte) = peek(take(1usize))(input)?;
    let first_byte = first_byte[0];

    let num_bytes = count_leading_zero_bits(first_byte) + 1;

    // IDs can only have up to 4 bytes
    if num_bytes > 4 {
        println!("found ID with more than 4 bytes");
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
    pub body_size: Option<u64>,
    #[serde(serialize_with = "serialize_size")]
    pub size: Option<u64>,
}

fn serialize_size<S: Serializer>(size: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
    if let Some(size) = size {
        s.serialize_u64(*size)
    } else {
        s.serialize_str("Unknown")
    }
}

impl Header {
    fn new(id: Id, header_size: usize, body_size: u64) -> Self {
        Self {
            id,
            header_size,
            body_size: Some(body_size),
            size: Some(header_size as u64 + body_size),
        }
    }

    fn with_uknown_size(id: Id, header_size: usize) -> Self {
        Self {
            id,
            header_size,
            body_size: None,
            size: None,
        }
    }
}

fn count_leading_zero_bits(input: u8) -> u8 {
    const MASK: u8 = 0b10000000;
    for leading_zeros in 0..8 {
        if input >= (MASK >> leading_zeros) {
            return leading_zeros;
        }
    }
    8
}

fn parse_varint(first_input: &[u8]) -> IResult<&[u8], Option<u64>> {
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
    let bitmask = (1 << num_bits_in_value) - 1;
    value &= bitmask;

    // If all VINT_DATA bits are set to 1, it's an unkown size/value
    // https://github.com/ietf-wg-cellar/ebml-specification/blob/master/specification.markdown#unknown-data-size
    let result = if value != bitmask { Some(value) } else { None };

    Ok((input, result))
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let initial_len = input.len();
    let (input, id) = parse_id(input)?;
    let (input, body_size) = parse_varint(input)?;

    if body_size.is_none() && id.get_type() != Type::Master {
        panic!("Unknown sizes are only supported in Master elements");
    }

    let header_size = initial_len - input.len();

    let header = if let Some(body_size) = body_size {
        Header::new(id, header_size, body_size)
    } else {
        Header::with_uknown_size(id, header_size)
    };

    Ok((input, header))
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
    #[serde(serialize_with = "serialize_short_payloads")]
    Standard(Vec<u8>),
    SeekId(Id),
    SimpleBlock(SimpleBlock),
    Block(Block),
    Void,
}

fn serialize_short_payloads<S: Serializer>(payload: &[u8], s: S) -> Result<S::Ok, S::Error> {
    const MAX_LENGTH: usize = 64;
    if payload.len() <= MAX_LENGTH {
        let string_values = payload
            .iter()
            .map(|n| format!("{:02x}", n))
            .fold("".to_owned(), |acc, s| acc + &s + " ")
            .trim_end()
            .to_owned();
        s.serialize_str(&format!("[{}]", string_values))
    } else {
        s.serialize_str(&format!("{} bytes", payload.len()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
enum Body {
    Master,
    Unsigned(Enumeration),
    Signed(i64),
    Float(f64),
    String(String),
    Utf8(String),
    Date(DateTime<Utc>),
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
        Type::Unsigned => parse_int(&header, input)
            .map(|(input, value)| (input, Body::Unsigned(Enumeration::new(&header.id, value)))),
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
        Type::Date => parse_date(&header, input).map(|(input, value)| (input, Body::Date(value))),
        Type::Binary => parse_binary(&header, input).map(|(input, value)| {
            let binary_value = match header.id {
                Id::SeekId => BinaryValue::SeekId(parse_id(&value).unwrap().1),
                Id::SimpleBlock => BinaryValue::SimpleBlock(parse_simple_block(&value).unwrap().1),
                Id::Block => BinaryValue::Block(parse_block(&value).unwrap().1),
                Id::Void => BinaryValue::Void,
                _ => BinaryValue::Standard(value),
            };
            (input, Body::Binary(binary_value))
        }),
    }?;
    let element = Element { header, body };
    Ok((input, element))
}

fn parse_string<'a>(metadata: &Header, input: &'a [u8]) -> IResult<&'a [u8], String> {
    let (input, string_bytes) =
        take(metadata.body_size.expect("Strings need a known body size"))(input)?;
    let value = String::from_utf8(string_bytes.to_vec())
        .map_err(|_| Err::Failure(Error::new(input, ErrorKind::Fail)))?;

    // Remove trimming null characters
    let value = value.trim_end_matches('\0').to_string();

    Ok((input, value))
}

fn parse_binary<'a>(metadata: &Header, input: &'a [u8]) -> IResult<&'a [u8], Vec<u8>> {
    let (input, bytes) = take(metadata.body_size.expect("Binaries need a known body size"))(input)?;
    let value = Vec::from(bytes);

    Ok((input, value))
}

fn parse_date<'a>(metadata: &Header, input: &'a [u8]) -> IResult<&'a [u8], DateTime<Utc>> {
    let (input, timestamp_nanos_to_2001) = parse_int::<i64>(metadata, input)?;
    let nanos_2001 = NaiveDate::from_ymd(2001, 1, 1)
        .and_hms(0, 0, 0)
        .timestamp_nanos();
    let timestamp_seconds_to_1970 = (timestamp_nanos_to_2001 + nanos_2001) / 1_000_000_000;
    Ok((
        input,
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(timestamp_seconds_to_1970, 0),
            Utc,
        ),
    ))
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
    let (input, int_bytes) =
        take(metadata.body_size.expect("Integers need a known body size"))(input)?;
    // FIXME: any efficient way to avoid this copy here?
    let mut value_buffer = [0u8; 8];
    value_buffer[(8 - int_bytes.len())..].copy_from_slice(int_bytes);
    let value = T::from_be_bytes(value_buffer);

    Ok((input, value))
}

fn parse_float<'a>(metadata: &Header, input: &'a [u8]) -> IResult<&'a [u8], f64> {
    let body_size = metadata.body_size.expect("Floats need a known body size");
    let (input, float_bytes) = take(body_size)(input)?;

    if body_size == 4 {
        let value = f32::from_be_bytes(float_bytes.try_into().unwrap()) as f64;
        Ok((input, value))
    } else if body_size == 8 {
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
    let track_number = track_number.expect("Blocks need a known track number");
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
    let track_number = track_number.expect("SimpleBlocks need a known track number");
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
                //
                let mut size_remaining = element.header.body_size.unwrap_or_else(|| match element
                    .header
                    .id
                {
                    // Only Segment and Cluster have unknownsizeallowed="1" in ebml_matroska.xml.
                    // Also mentioned in https://www.w3.org/TR/mse-byte-stream-format-webm/
                    // FIXME (#13): this u64::MAX hack would not be needed if we built the tree up from
                    // the element paths, rather, than relying on the sizes as we do now.
                    Id::Segment | Id::Cluster => u64::MAX,
                    _ => panic!("Only Segment or Cluster elements can have unknown sizes"),
                });

                // TODO: no need to copy here
                let mut children = Vec::<Element>::new();
                while size_remaining > 0 {
                    index += 1;
                    if let Some(next_child) = elements.get(index) {
                        // Hack before building tree using XML paths (#13):
                        // a Cluster can't be parent of another Cluster
                        if element.header.id == Id::Cluster && next_child.header.id == Id::Cluster {
                            index -= 1;
                            break;
                        }

                        size_remaining -= if let Body::Master = next_child.body {
                            // Master elements' body size should not count in the recursion
                            // as the children would duplicate the size count, so
                            // we only consider the header size on the calculation.
                            next_child.header.header_size as u64
                        } else {
                            next_child
                                .header
                                .size
                                .expect("Only Master elements can have unknown size")
                        };
                        children.push(next_child.clone());
                    } else {
                        // Elements have ended before reaching the size of the master element
                        break;
                    }
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

fn parse_buffer_to_end(input: &[u8]) -> Vec<ElementTree> {
    let mut elements = Vec::<Element>::new();
    let mut read_buffer = input;
    loop {
        match parse_element(read_buffer) {
            Ok((new_read_buffer, element)) => {
                elements.push(element);
                if new_read_buffer.is_empty() {
                    break;
                }
                read_buffer = new_read_buffer;
            }
            _ => {
                println!("skipping one byte");
                read_buffer = &read_buffer[1..];
                if read_buffer.is_empty() {
                    break;
                }
            }
        }
    }
    build_element_trees(&elements)
}

fn print_element_trees(element_trees: &[ElementTree], format: &str) {
    let serialized = if format == "json" {
        serde_json::to_string_pretty(element_trees).unwrap()
    } else {
        serde_yaml::to_string(element_trees).unwrap()
    };
    println!("{}", serialized);
}

#[cfg(test)]
mod tests {
    use nom::Needed;

    use crate::enumerations::TrackType;

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
        assert_eq!(parse_varint(&[0x9F]), Ok((EMPTY, Some(31))));
        assert_eq!(parse_varint(&[0x81]), Ok((EMPTY, Some(1))));
        assert_eq!(parse_varint(&[0x53, 0xAC]), Ok((EMPTY, Some(5036))));

        const INVALID_VARINT: &[u8] = &[0x00, 0xAC];
        assert_eq!(
            parse_varint(INVALID_VARINT),
            Err(Err::Failure(Error::new(INVALID_VARINT, ErrorKind::Fail)))
        );

        const UNKNOWN_VARINT: &[u8] = &[0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        assert_eq!(parse_varint(UNKNOWN_VARINT), Ok((EMPTY, None)));
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
        );

        assert_eq!(
            parse_string(
                &Header::new(Id::DocType, 3, 6),
                &[0x77, 0x65, 0x62, 0x6D, 0x00, 0x00]
            ),
            Ok((EMPTY, "webm".to_string()))
        );
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
    fn test_parse_date() {
        let expected_datetime =
            DateTime::<Utc>::from_utc(NaiveDate::from_ymd(2022, 8, 11).and_hms(8, 27, 15), Utc);
        assert_eq!(
            parse_date(
                &Header::new(Id::DateUtc, 1, 8),
                &[0x09, 0x76, 0x97, 0xbd, 0xca, 0xc9, 0x1e, 0x00]
            ),
            Ok((EMPTY, expected_datetime))
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
    fn test_parse_enumeration() {
        const INPUT: &[u8] = &[0x83, 0x81, 0x01];
        assert_eq!(
            parse_element(INPUT),
            Ok((
                EMPTY,
                Element {
                    header: Header::new(Id::TrackType, 2, 1),
                    body: Body::Unsigned(Enumeration::TrackType(TrackType::Video))
                }
            ))
        );

        const INPUT_UNKNOWN_ENUMERATION: &[u8] = &[0x83, 0x81, 0xFF];
        assert_eq!(
            parse_element(INPUT_UNKNOWN_ENUMERATION),
            Ok((
                EMPTY,
                Element {
                    header: Header::new(Id::TrackType, 2, 1),
                    body: Body::Unsigned(Enumeration::Unknown(255))
                }
            ))
        );
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
                    body: Body::Binary(BinaryValue::Standard(vec![0xAF, 0x93, 0x97, 0x18]))
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
                body: Body::Unsigned(1.into()),
            },
            Element {
                header: Header::new(Id::EbmlReadVersion, 3, 1),
                body: Body::Unsigned(1.into()),
            },
            Element {
                header: Header::new(Id::EbmlMaxIdLength, 3, 1),
                body: Body::Unsigned(4.into()),
            },
            Element {
                header: Header::new(Id::EbmlMaxSizeLength, 3, 1),
                body: Body::Unsigned(8.into()),
            },
            Element {
                header: Header::new(Id::DocType, 3, 4),
                body: Body::String("webm".to_string()),
            },
            Element {
                header: Header::new(Id::DocTypeVersion, 3, 1),
                body: Body::Unsigned(4.into()),
            },
            Element {
                header: Header::new(Id::DocTypeReadVersion, 3, 1),
                body: Body::Unsigned(2.into()),
            },
        ];

        let expected = vec![ElementTree::Master(MasterElement {
            header: Header::new(Id::Ebml, 5, 31),
            children: vec![
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlVersion, 3, 1),
                    body: Body::Unsigned(1.into()),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlReadVersion, 3, 1),
                    body: Body::Unsigned(1.into()),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlMaxIdLength, 3, 1),
                    body: Body::Unsigned(4.into()),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::EbmlMaxSizeLength, 3, 1),
                    body: Body::Unsigned(8.into()),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::DocType, 3, 4),
                    body: Body::String("webm".to_string()),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::DocTypeVersion, 3, 1),
                    body: Body::Unsigned(4.into()),
                }),
                ElementTree::Normal(Element {
                    header: Header::new(Id::DocTypeReadVersion, 3, 1),
                    body: Body::Unsigned(2.into()),
                }),
            ],
        })];

        assert_eq!(build_element_trees(&elements), expected);
    }

    #[test]
    fn test_binary_custom_serializer() {
        let binary_value = BinaryValue::Standard(vec![1, 2, 3]);
        assert_eq!(
            serde_yaml::to_string(&binary_value).unwrap().trim(),
            "'[01 02 03]'"
        );

        let binary_value = BinaryValue::Standard(vec![0; 65]);
        assert_eq!(
            serde_yaml::to_string(&binary_value).unwrap().trim(),
            "65 bytes"
        );
    }

    #[test]
    fn test_serialize_enumeration() {
        assert_eq!(
            serde_yaml::to_string(&Enumeration::TrackType(TrackType::Video))
                .unwrap()
                .trim(),
            "video"
        );
        assert_eq!(
            serde_yaml::to_string(&Enumeration::Unknown(5u64))
                .unwrap()
                .trim(),
            "5"
        );
    }

    macro_rules! snapshot_test {
        ($test_name:ident, $filename:expr) => {
            #[test]
            fn $test_name() {
                insta::assert_yaml_snapshot!(parse_buffer_to_end(include_bytes!($filename)));
            }
        };
    }

    snapshot_test!(
        test_parse_incomplete_file_should_not_panic,
        "../inputs/incomplete.hdr"
    );
    snapshot_test!(test_parse_header_encrypted, "../inputs/encrypted.hdr");

    // File was generated with:
    // ffmpeg -f lavfi -i testsrc -c:v libx264 -frames:v 2 -metadata creation_time="2022-08-11T08:27:15Z" -f matroska test.mkv
    snapshot_test!(test_parse_file_with_dateutc, "../inputs/dateutc.mkv");

    // Tests from Matroska test suite
    // TODO(#25): fix tests for files 4 and 7
    snapshot_test!(test1, "../inputs/matroska-test-suite/test1.mkv");
    snapshot_test!(test2, "../inputs/matroska-test-suite/test2.mkv");
    snapshot_test!(test3, "../inputs/matroska-test-suite/test3.mkv");
    snapshot_test!(test4, "../inputs/matroska-test-suite/test4.mkv");
    snapshot_test!(test5, "../inputs/matroska-test-suite/test5.mkv");
    snapshot_test!(test6, "../inputs/matroska-test-suite/test6.mkv");
    // snapshot_test!(test7, "../inputs/matroska-test-suite/test7.mkv");
    snapshot_test!(test8, "../inputs/matroska-test-suite/test8.mkv");
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

    // TODO(#8): read chunked to not load entire file in memory.
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;

    let element_trees = parse_buffer_to_end(&buffer);

    print_element_trees(&element_trees, &args.format);

    Ok(())
}
