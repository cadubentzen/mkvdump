use std::ops::Not;

use chrono::prelude::*;
use nom::combinator::peek;
use nom::ToUsize;
use serde::{Serialize, Serializer};
use serde_with::skip_serializing_none;

mod ebml;
mod elements;
pub mod enumerations;
mod error;

pub use crate::elements::{Id, Type};
pub use crate::enumerations::Enumeration;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
type IResult<T, O> = Result<(T, O)>;

fn take<'a>(
    len: impl ToUsize,
) -> impl Fn(&'a [u8]) -> std::result::Result<(&'a [u8], &'a [u8]), nom::Err<()>> {
    nom::bytes::streaming::take(len)
}

fn parse_id(input: &[u8]) -> IResult<&[u8], Id> {
    let (input, first_byte) = peek(take(1usize))(input)?;
    let first_byte = first_byte[0];

    let num_bytes = count_leading_zero_bits(first_byte) + 1;

    // IDs can only have up to 4 bytes in Matroska
    if num_bytes > 4 {
        return Err(Error::InvalidId);
    }

    let (input, varint_bytes) = take(num_bytes)(input)?;
    let mut value_buffer = [0u8; 4];
    value_buffer[(4 - varint_bytes.len())..].copy_from_slice(varint_bytes);
    let id = u32::from_be_bytes(value_buffer);

    Ok((input, Id::new(id)))
}

#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Header {
    pub id: Id,
    pub header_size: usize,
    #[serde(skip_serializing)]
    pub body_size: Option<usize>,
    #[serialize_always]
    #[serde(serialize_with = "serialize_size")]
    pub size: Option<usize>,
    pub position: Option<usize>,
}

fn serialize_size<S: Serializer>(
    size: &Option<usize>,
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
    if let Some(size) = size {
        s.serialize_u64(*size as u64)
    } else {
        s.serialize_str("Unknown")
    }
}

impl Header {
    fn new(id: Id, header_size: usize, body_size: usize) -> Self {
        Self {
            id,
            header_size,
            body_size: Some(body_size),
            size: Some(header_size + body_size),
            position: None,
        }
    }

    fn with_unknown_size(id: Id, header_size: usize) -> Self {
        Self {
            id,
            header_size,
            body_size: None,
            size: None,
            position: None,
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

fn parse_varint(first_input: &[u8]) -> IResult<&[u8], Option<usize>> {
    let (input, first_byte) = peek(take(1usize))(first_input)?;
    let first_byte = first_byte[0];

    let vint_prefix_size = count_leading_zero_bits(first_byte) + 1;

    // Maximum 8 bytes, i.e. first byte can't be 0
    if vint_prefix_size > 8 {
        return Err(Error::InvalidVarint);
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
    //
    // In 32-bit plaforms, the conversion from u64 to usize will fail if the value
    // is bigger than u32::MAX.
    let result = (value != bitmask).then_some(value.try_into()?);

    Ok((input, result))
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let initial_len = input.len();
    let (input, id) = parse_id(input)?;
    let (input, body_size) = parse_varint(input)?;

    // Only Segment and Cluster have unknownsizeallowed="1" in ebml_matroska.xml.
    // Also mentioned in https://www.w3.org/TR/mse-byte-stream-format-webm/
    if body_size.is_none() && id != Id::Segment && id != Id::Cluster {
        return Err(Error::ForbiddenUnknownSize);
    }

    let header_size = initial_len - input.len();

    let header = match body_size {
        Some(body_size) => Header::new(id, header_size, body_size),
        None => Header::with_unknown_size(id, header_size),
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
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Block {
    track_number: usize,
    timestamp: i16,
    #[serde(skip_serializing_if = "Not::not")]
    invisible: bool,
    lacing: Option<Lacing>,
    num_frames: Option<u8>,
}

// https://www.matroska.org/technical/basics.html#simpleblock-structure
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SimpleBlock {
    track_number: usize,
    timestamp: i16,
    #[serde(skip_serializing_if = "Not::not")]
    keyframe: bool,
    #[serde(skip_serializing_if = "Not::not")]
    invisible: bool,
    lacing: Option<Lacing>,
    #[serde(skip_serializing_if = "Not::not")]
    discardable: bool,
    num_frames: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum BinaryValue {
    #[serde(serialize_with = "serialize_short_payloads")]
    Standard(Vec<u8>),
    SeekId(Id),
    SimpleBlock(SimpleBlock),
    Block(Block),
    Void,
    Corrupted,
}

impl BinaryValue {
    fn new(id: &Id, value: &[u8]) -> Result<Self> {
        Ok(match id {
            Id::SeekId => BinaryValue::SeekId(parse_id(&value)?.1),
            Id::SimpleBlock => BinaryValue::SimpleBlock(parse_simple_block(&value)?.1),
            Id::Block => BinaryValue::Block(parse_block(&value)?.1),
            Id::Void => BinaryValue::Void,
            _ => BinaryValue::Standard(value.into()),
        })
    }
}

fn serialize_short_payloads<S: Serializer>(
    payload: &[u8],
    s: S,
) -> std::result::Result<S::Ok, S::Error> {
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
pub enum Body {
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
pub struct Element {
    #[serde(flatten)]
    pub header: Header,
    #[serde(rename = "value")]
    pub body: Body,
}

// If we ever hit a damaged element, we'll try to recover by finding
// one of those IDs next to start clean. Those are the 4-bytes IDs,
// which according to the EBML spec:
// "Four-octet Element IDs are somewhat special in that they are useful
// for resynchronizing to major structures in the event of data corruption or loss."
const SYNC_ELEMENT_IDS: &[Id] = &[
    Id::Cluster,
    Id::Ebml,
    Id::Segment,
    Id::SeekHead,
    Id::Info,
    Id::Tracks,
    Id::Cues,
    Id::Attachments,
    Id::Chapters,
    Id::Tags,
];

fn find_valid_element(input: &[u8]) -> IResult<&[u8], Element> {
    const SYNC_ID_LEN: usize = 4;
    for (offset, window) in input.windows(SYNC_ID_LEN).enumerate() {
        for sync_id in SYNC_ELEMENT_IDS {
            let id_value = sync_id.get_value().unwrap();
            let id_bytes = id_value.to_be_bytes();
            if window == id_bytes {
                return Ok((
                    &input[offset..],
                    Element {
                        header: Header::new(Id::corrupted(), offset, 0),
                        body: Body::Binary(BinaryValue::Corrupted),
                    },
                ));
            }
        }
    }
    Err(Error::ValidElementNotFound)
}

fn parse_element(original_input: &[u8]) -> IResult<&[u8], Element> {
    let (input, header) = parse_header(original_input)?;
    let element_type = header.id.get_type();

    let (input, body) = match element_type {
        Type::Master => (input, Body::Master),
        Type::Unsigned => {
            let (input, value) = parse_int(&header, input)?;
            (input, Body::Unsigned(Enumeration::new(&header.id, value)))
        }
        Type::Signed => {
            let (input, value) = parse_int(&header, input)?;
            (input, Body::Signed(value))
        }
        Type::Float => {
            let (input, value) = parse_float(&header, input)?;
            (input, Body::Float(value))
        }
        Type::String => {
            let (input, value) = parse_string(&header, input)?;
            (input, Body::String(value))
        }
        Type::Utf8 => {
            let (input, value) = parse_string(&header, input)?;
            (input, Body::Utf8(value))
        }
        Type::Date => {
            let (input, value) = parse_date(&header, input)?;
            (input, Body::Date(value))
        }
        Type::Binary => {
            let (input, value) = parse_binary(&header, input)?;
            (input, Body::Binary(BinaryValue::new(&header.id, value)?))
        }
    };

    let element = Element { header, body };
    Ok((input, element))
}

fn parse_string<'a>(header: &Header, input: &'a [u8]) -> IResult<&'a [u8], String> {
    let body_size = header.body_size.ok_or(Error::ForbiddenUnknownSize)?;
    let (input, string_bytes) = take(body_size)(input)?;
    let value = String::from_utf8(string_bytes.to_vec())?;

    // Remove trimming null characters
    let value = value.trim_end_matches('\0').to_string();

    Ok((input, value))
}

fn parse_binary<'a>(header: &Header, input: &'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    let body_size = header.body_size.ok_or(Error::ForbiddenUnknownSize)?;

    Ok(take(body_size)(input)?)
}

fn parse_date<'a>(header: &Header, input: &'a [u8]) -> IResult<&'a [u8], DateTime<Utc>> {
    let (input, timestamp_nanos_to_2001) = parse_int::<i64>(header, input)?;
    let nanos_2001 = NaiveDate::from_ymd_opt(2001, 1, 1)
        .ok_or(Error::InvalidDate)?
        .and_hms_opt(0, 0, 0)
        .ok_or(Error::InvalidDate)?
        .timestamp_nanos();
    let timestamp_seconds_to_1970 = (timestamp_nanos_to_2001 + nanos_2001) / 1_000_000_000;
    Ok((
        input,
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(timestamp_seconds_to_1970, 0)
                .ok_or(Error::InvalidDate)?,
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
    header: &Header,
    input: &'a [u8],
) -> IResult<&'a [u8], T> {
    let body_size = header.body_size.ok_or(Error::ForbiddenUnknownSize)?;
    if body_size > 8 {
        return Err(Error::ForbiddenIntegerSize);
    }

    let (input, int_bytes) = take(body_size)(input)?;

    let mut value_buffer = [0u8; 8];
    value_buffer[(8 - int_bytes.len())..].copy_from_slice(int_bytes);
    let value = T::from_be_bytes(value_buffer);

    Ok((input, value))
}

fn parse_float<'a>(header: &Header, input: &'a [u8]) -> IResult<&'a [u8], f64> {
    let body_size = header.body_size.ok_or(Error::ForbiddenUnknownSize)?;

    if body_size == 4 {
        let (input, float_bytes) = take(body_size)(input)?;
        let value = f32::from_be_bytes(float_bytes.try_into().unwrap()) as f64;
        Ok((input, value))
    } else if body_size == 8 {
        let (input, float_bytes) = take(body_size)(input)?;
        let value = f64::from_be_bytes(float_bytes.try_into().unwrap());
        Ok((input, value))
    } else if body_size == 0 {
        Ok((input, 0f64))
    } else {
        Err(Error::ForbiddenFloatSize)
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
        0b01 => Some(Lacing::Xiph),
        0b11 => Some(Lacing::Ebml),
        0b10 => Some(Lacing::FixedSize),
        _ => None,
    }
}

fn parse_block(input: &[u8]) -> IResult<&[u8], Block> {
    let (input, track_number) = parse_varint(input)?;
    let track_number = track_number.ok_or(Error::MissingTrackNumber)?;
    let (input, timestamp) = parse_i16(input)?;
    let (input, flags) = take(1usize)(input)?;
    let flags = flags[0];

    let invisible = is_invisible(flags);
    let lacing = get_lacing(flags);
    let (input, num_frames) = if lacing.is_some() {
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
    let track_number = track_number.ok_or(Error::MissingTrackNumber)?;
    let (input, timestamp) = parse_i16(input)?;
    let (input, flags) = take(1usize)(input)?;
    let flags = flags[0];

    let keyframe = (flags & (1 << 7)) != 0;
    let invisible = is_invisible(flags);
    let lacing = get_lacing(flags);
    let discardable = (flags & 0b1) != 0;
    let (input, num_frames) = if lacing.is_some() {
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
pub struct MasterElement {
    #[serde(flatten)]
    header: Header,
    children: Vec<ElementTree>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ElementTree {
    Normal(Element),
    Master(MasterElement),
}

impl Id {
    fn can_be_children_of(&self, other: &Id) -> bool {
        !matches!((self, other), (Id::Cluster, Id::Cluster) | (Id::Ebml, _))
    }
}

fn build_element_trees(elements: &[Element]) -> Vec<ElementTree> {
    let mut trees = Vec::<ElementTree>::new();

    let mut index = 0;
    while index < elements.len() {
        let element = &elements[index];
        match element.body {
            Body::Master => {
                // parse_header() already handles Unknown sizes.
                let mut size_remaining = element.header.body_size.unwrap_or(usize::MAX);

                let mut children = Vec::<Element>::new();
                while size_remaining > 0 {
                    index += 1;

                    if let Some(next_child) = elements.get(index) {
                        if !next_child.header.id.can_be_children_of(&element.header.id) {
                            index -= 1;
                            break;
                        }

                        size_remaining -= if let Body::Master = next_child.body {
                            // Master elements' body size should not count in the recursion
                            // as the children would duplicate the size count, so
                            // we only consider the header size on the calculation.
                            next_child.header.header_size
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

fn parse_element_or_skip_corrupted(input: &[u8]) -> IResult<&[u8], Element> {
    parse_element(input).or_else(|_| find_valid_element(input))
}

pub fn parse_elements(input: &[u8], show_position: bool) -> Vec<Element> {
    let mut elements = Vec::<Element>::new();
    let mut read_buffer = input;
    let mut position = show_position.then_some(0);

    while let Ok((new_read_buffer, mut element)) = parse_element_or_skip_corrupted(read_buffer) {
        element.header.position = position;
        position = position.map(|p| {
            if let Body::Master = element.body {
                p + element.header.header_size
            } else {
                // It's safe to unwrap because all non-Master elements have a set size
                p + element.header.size.unwrap()
            }
        });
        elements.push(element);
        if new_read_buffer.is_empty() {
            break;
        }
        read_buffer = new_read_buffer;
    }
    elements
}

pub fn parse_buffer_to_end(input: &[u8], show_position: bool) -> Vec<ElementTree> {
    let elements = parse_elements(input, show_position);
    build_element_trees(&elements)
}

#[cfg(test)]
mod tests {
    use crate::enumerations::TrackType;

    use super::*;

    const EMPTY: &[u8] = &[];
    const UNKNOWN_VARINT: &[u8] = &[0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

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
            Err(Error::Parsing(nom::Err::Incomplete(nom::Needed::Size(
                1.try_into().unwrap()
            ))))
        );

        // Longer than 4 bytes
        const FAILURE_INPUT: &[u8] = &[0x08, 0x45, 0xDF, 0xA3];
        assert_eq!(parse_id(FAILURE_INPUT), Err(Error::InvalidId));

        // Unknown ID
        let (remaining, id) = parse_id(&[0x19, 0xAB, 0xCD, 0xEF]).unwrap();
        assert_eq!((remaining, &id), (EMPTY, &Id::Unknown(0x19ABCDEF)));
        assert_eq!(serde_yaml::to_string(&id).unwrap().trim(), "'0x19ABCDEF'");
        assert_eq!(id.get_value().unwrap(), 0x19ABCDEF);
    }

    #[test]
    fn test_parse_varint() {
        assert_eq!(parse_varint(&[0x9F]), Ok((EMPTY, Some(31))));
        assert_eq!(parse_varint(&[0x81]), Ok((EMPTY, Some(1))));
        assert_eq!(parse_varint(&[0x53, 0xAC]), Ok((EMPTY, Some(5036))));

        const INVALID_VARINT: &[u8] = &[0x00, 0xAC];
        assert_eq!(parse_varint(INVALID_VARINT), Err(Error::InvalidVarint));

        assert_eq!(parse_varint(UNKNOWN_VARINT), Ok((EMPTY, None)));
    }

    #[test]
    fn test_parse_element_header() {
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

        assert_eq!(
            parse_string(&Header::with_unknown_size(Id::DocType, 3), EMPTY),
            Err(Error::ForbiddenUnknownSize)
        );
    }

    #[test]
    fn test_parse_corrupted() {
        // This integer would have more than 8 bytes.
        // It needs to find a valid 4-byte Element ID, but can't
        // so we get an incomplete.
        assert_eq!(
            parse_element(&[0x42, 0x87, 0x90, 0x01]),
            Err(Error::ForbiddenIntegerSize)
        );

        // Now it finds a Segment.
        const SEGMENT_ID: &[u8] = &[0x18, 0x53, 0x80, 0x67];
        let (remaining, element) =
            parse_element_or_skip_corrupted(&[0x42, 0x87, 0x90, 0x01, 0x18, 0x53, 0x80, 0x67])
                .unwrap();
        assert_eq!(
            (remaining, &element),
            (
                SEGMENT_ID,
                &Element {
                    header: Header::new(Id::corrupted(), 4, 0),
                    body: Body::Binary(BinaryValue::Corrupted),
                },
            )
        );
        assert!(element.header.id.get_value().is_none());
    }

    #[test]
    fn test_parse_corrupted_unknown_size() {
        // String
        assert_eq!(
            parse_element(&[0x86, 0xFF, 0x56, 0x5F, 0x54]),
            Err(Error::ForbiddenUnknownSize)
        );

        // Binary
        assert_eq!(
            parse_element(&[0x63, 0xA2, 0xFF]),
            Err(Error::ForbiddenUnknownSize)
        );

        // Integer
        assert_eq!(
            parse_element(&[0x42, 0x87, 0xFF, 0x01]),
            Err(Error::ForbiddenUnknownSize)
        );

        // Float
        assert_eq!(
            parse_element(&[0x44, 0x89, 0xFF, 0x01]),
            Err(Error::ForbiddenUnknownSize)
        );
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(
            parse_int(&Header::new(Id::EbmlVersion, 3, 1), &[0x01]),
            Ok((EMPTY, 1u64))
        );
        assert_eq!(
            parse_int::<u64>(&Header::with_unknown_size(Id::EbmlVersion, 3), EMPTY),
            Err(Error::ForbiddenUnknownSize)
        );
        assert_eq!(
            parse_int::<i64>(&Header::with_unknown_size(Id::EbmlVersion, 3), EMPTY),
            Err(Error::ForbiddenUnknownSize)
        );
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(
            parse_float(&Header::new(Id::Duration, 3, 4), &[0x45, 0x7A, 0x30, 0x00]),
            Ok((EMPTY, 4003.))
        );
        assert_eq!(
            parse_float(
                &Header::new(Id::Duration, 3, 8),
                &[0x40, 0xAF, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00]
            ),
            Ok((EMPTY, 4003.))
        );
        assert_eq!(
            parse_float(&Header::new(Id::Duration, 3, 0), EMPTY),
            Ok((EMPTY, 0.))
        );
        assert_eq!(
            parse_float(&Header::new(Id::Duration, 3, 7), EMPTY),
            Err(Error::ForbiddenFloatSize)
        );
        assert_eq!(
            parse_float(&Header::with_unknown_size(Id::Duration, 3), EMPTY),
            Err(Error::ForbiddenUnknownSize)
        );
    }

    #[test]
    fn test_parse_binary() {
        const BODY: &[u8] = &[0x15, 0x49, 0xA9, 0x66];
        assert_eq!(
            parse_binary(&Header::new(Id::SeekId, 3, 4), BODY),
            Ok((EMPTY, BODY))
        );
        assert_eq!(
            parse_binary(&Header::with_unknown_size(Id::SeekId, 3), EMPTY),
            Err(Error::ForbiddenUnknownSize)
        );
    }

    #[test]
    fn test_parse_date() {
        let expected_datetime = DateTime::<Utc>::from_utc(
            NaiveDate::from_ymd_opt(2022, 8, 11)
                .unwrap()
                .and_hms_opt(8, 27, 15)
                .unwrap(),
            Utc,
        );
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
        let (remaining, element) = parse_element(INPUT_UNKNOWN_ENUMERATION).unwrap();
        assert_eq!(
            (remaining, &element),
            (
                EMPTY,
                &Element {
                    header: Header::new(Id::TrackType, 2, 1),
                    body: Body::Unsigned(Enumeration::Unknown(255))
                }
            )
        );
        assert_eq!(
            serde_yaml::to_string(&element).unwrap().trim(),
            "id: TrackType\nheader_size: 2\nsize: 3\nvalue: 255"
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
        );

        assert_eq!(parse_block(UNKNOWN_VARINT), Err(Error::MissingTrackNumber));
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
        );

        assert_eq!(
            parse_simple_block(UNKNOWN_VARINT),
            Err(Error::MissingTrackNumber)
        );
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

    #[test]
    fn test_show_position() {
        const INPUT: &[u8] = include_bytes!("../inputs/matroska-test-suite/test7.mkv");
        let elements = parse_elements(INPUT, true);
        for element in elements {
            // Corrupted elements won't match as we ignore their ID due to invalid content.
            if element.header.id == Id::Corrupted {
                continue;
            }
            let (_, id_at_position) =
                parse_id(&INPUT[element.header.position.unwrap() as usize..]).unwrap();
            assert_eq!(id_at_position, element.header.id);
        }
    }

    #[test]
    fn test_find_valid_element() {
        // impossible to find in an empty array
        assert_eq!(find_valid_element(&[]), Err(Error::ValidElementNotFound));
        // can not find in a bonkers array
        assert_eq!(
            find_valid_element(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            Err(Error::ValidElementNotFound)
        );
    }
}
