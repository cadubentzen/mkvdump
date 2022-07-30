use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};

use clap::Parser;
use enum_primitive_derive::Primitive;
use lazy_static::lazy_static;
use nom::{
    bytes::streaming::take,
    combinator::peek,
    error::{Error, ErrorKind},
    Err, IResult,
};
use num_traits::FromPrimitive;
use serde::Serialize;

#[derive(Debug, PartialEq)]
enum Type {
    Unsigned,
    Signed,
    Float,
    String,
    Utf8,
    Date,
    Master,
    Binary,
}

#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Primitive, Copy, Eq, Hash, Serialize)]
pub enum Id {
    Unknown = 0x00,
    Ebml = 0x1A45DFA3,
    EbmlVersion = 0x4286,
    EbmlReadVersion = 0x42F7,
    EbmlMaxIdLength = 0x42F2,
    EbmlMaxSizeLength = 0x42F3,
    DocType = 0x4282,
    DocTypeVersion = 0x4287,
    DocTypeReadVersion = 0x4285,
    Void = 0xEC,
    Segment = 0x18538067,
    SeekHead = 0x114D9B74,
    Seek = 0x4DBB,
    SeekId = 0x53AB,
    SeekPosition = 0x53AC,
    Info = 0x1549A966,
    TimecodeScale = 0x2AD7B1,
    Duration = 0x4489,
    DateUtc = 0x4461,
    Title = 0x7BA9,
    MuxingApp = 0x4D80,
    WritingApp = 0x5741,
    Cluster = 0x1F43B675,
    Timecode = 0xE7,
    PrevSize = 0xAB,
    SimpleBlock = 0xA3,
    BlockGroup = 0xA0,
    Block = 0xA1,
    BlockVirtual = 0xA2,
    BlockAdditions = 0x75A1,
    BlockMore = 0xA6,
    BlockAddId = 0xEE,
    BlockAdditional = 0xA5,
    BlockDuration = 0x9B,
    ReferenceBlock = 0xFB,
    DiscardPadding = 0x75A2,
    Slices = 0x8E,
    TimeSlice = 0xE8,
    LaceNumber = 0xCC,
    Tracks = 0x1654AE6B,
    TrackEntry = 0xAE,
    TrackNumber = 0xD7,
    TrackUid = 0x73C5,
    TrackType = 0x83,
    FlagEnabled = 0xB9,
    FlagDefault = 0x88,
    FlagForced = 0x55AA,
    FlagLacing = 0x9C,
    DefaultDuration = 0x23E383,
    Name = 0x536E,
    Language = 0x22B59C,
    CodecId = 0x86,
    CodecPrivate = 0x63A2,
    CodecName = 0x258688,
    CodecDelay = 0x56AA,
    SeekPreRoll = 0x56BB,
    Video = 0xE0,
    FlagInterlaced = 0x9A,
    StereoMode = 0x53B8,
    AlphaMode = 0x53C0,
    PixelWidth = 0xB0,
    PixelHeight = 0xBA,
    PixelCropBottom = 0x54AA,
    PixelCropTop = 0x54BB,
    PixelCropLeft = 0x54CC,
    PixelCropRight = 0x54DD,
    DisplayWidth = 0x54B0,
    DisplayHeight = 0x54BA,
    DisplayUnit = 0x54B2,
    AspectRatioType = 0x54B3,
    FrameRate = 0x2383E3,
    Colour = 0x55B0,
    MatrixCoefficients = 0x55B1,
    BitsPerChannel = 0x55B2,
    ChromaSubsamplingHorz = 0x55B3,
    ChromaSubsamplingVert = 0x55B4,
    CbSubsamplingHorz = 0x55B5,
    CbSubsamplingVert = 0x55B6,
    ChromaSitingHorz = 0x55B7,
    ChromaSitingVert = 0x55B8,
    Range = 0x55B9,
    TransferCharacteristics = 0x55BA,
    Primaries = 0x55BB,
    MaxCll = 0x55BC,
    MaxFall = 0x55BD,
    MasteringMetadata = 0x55D0,
    PrimaryRChromaticityX = 0x55D1,
    PrimaryRChromaticityY = 0x55D2,
    PrimaryGChromaticityX = 0x55D3,
    PrimaryGChromaticityY = 0x55D4,
    PrimaryBChromaticityX = 0x55D5,
    PrimaryBChromaticityY = 0x55D6,
    WhitePointChromaticityX = 0x55D7,
    WhitePointChromaticityY = 0x55D8,
    LuminanceMax = 0x55D9,
    LuminanceMin = 0x55DA,
    Projection = 0x7670,
    ProjectionType = 0x7671,
    ProjectionPrivate = 0x7672,
    ProjectionPoseYaw = 0x7673,
    ProjectionPosePitch = 0x7674,
    ProjectionPoseRoll = 0x7675,
    Audio = 0xE1,
    SamplingFrequency = 0xB5,
    OutputSamplingFrequency = 0x78B5,
    Channels = 0x9F,
    BitDepth = 0x6264,
    ContentEncodings = 0x6D80,
    ContentEncoding = 0x6240,
    ContentEncodingOrder = 0x5031,
    ContentEncodingScope = 0x5032,
    ContentEncodingType = 0x5033,
    ContentEncryption = 0x5035,
    ContentEncAlgo = 0x47E1,
    ContentEncKeyId = 0x47E2,
    ContentEncAesSettings = 0x47E7,
    AesSettingsCipherMode = 0x47E8,
    Cues = 0x1C53BB6B,
    CuePoint = 0xBB,
    CueTime = 0xB3,
    CueTrackPositions = 0xB7,
    CueTrack = 0xF7,
    CueClusterPosition = 0xF1,
    CueRelativePosition = 0xF0,
    CueDuration = 0xB2,
    CueBlockNumber = 0x5378,
    Chapters = 0x1043A770,
    EditionEntry = 0x45B9,
    ChapterAtom = 0xB6,
    ChapterUid = 0x73C4,
    ChapterStringUid = 0x5654,
    ChapterTimeStart = 0x91,
    ChapterTimeEnd = 0x92,
    ChapterDisplay = 0x80,
    ChapString = 0x85,
    ChapLanguage = 0x437C,
    ChapCountry = 0x437E,
    Tags = 0x1254C367,
    Tag = 0x7373,
    Targets = 0x63C0,
    TargetTypeValue = 0x68CA,
    TargetType = 0x63CA,
    TagTrackUid = 0x63C5,
    SimpleTag = 0x67C8,
    TagName = 0x45A3,
    TagLanguage = 0x447A,
    TagDefault = 0x4484,
    TagString = 0x4487,
    TagBinary = 0x4485,
}

// http://www.webmproject.org/docs/container/
// http://www.webmproject.org/docs/webm-encryption/#42-new-matroskawebm-elements
// http://matroska.org/technical/specs/index.html
// TODO: use proc-macros to avoid this lazy_static
lazy_static! {
    static ref ID_ELEMENT_TYPE_MAP: HashMap<Id, Type> = HashMap::from([
        (Id::Unknown, Type::Binary),
        (Id::Ebml, Type::Master),
        (Id::EbmlVersion, Type::Unsigned),
        (Id::EbmlReadVersion, Type::Unsigned),
        (Id::EbmlMaxIdLength, Type::Unsigned),
        (Id::EbmlMaxSizeLength, Type::Unsigned),
        (Id::DocType, Type::String),
        (Id::DocTypeVersion, Type::Unsigned),
        (Id::DocTypeReadVersion, Type::Unsigned),
        (Id::Void, Type::Binary),
        (Id::Segment, Type::Master),
        (Id::SeekHead, Type::Master),
        (Id::Seek, Type::Master),
        (Id::SeekId, Type::Binary),
        (Id::SeekPosition, Type::Unsigned),
        (Id::Info, Type::Master),
        (Id::TimecodeScale, Type::Unsigned),
        (Id::Duration, Type::Float),
        (Id::DateUtc, Type::Date),
        (Id::Title, Type::Utf8),
        (Id::MuxingApp, Type::Utf8),
        (Id::WritingApp, Type::Utf8),
        (Id::Cluster, Type::Master),
        (Id::Timecode, Type::Unsigned),
        (Id::PrevSize, Type::Unsigned),
        (Id::SimpleBlock, Type::Binary),
        (Id::BlockGroup, Type::Master),
        (Id::Block, Type::Binary),
        (Id::BlockVirtual, Type::Binary),
        (Id::BlockAdditions, Type::Master),
        (Id::BlockMore, Type::Master),
        (Id::BlockAddId, Type::Unsigned),
        (Id::BlockAdditional, Type::Binary),
        (Id::BlockDuration, Type::Unsigned),
        (Id::ReferenceBlock, Type::Signed),
        (Id::DiscardPadding, Type::Signed),
        (Id::Slices, Type::Master),
        (Id::TimeSlice, Type::Master),
        (Id::LaceNumber, Type::Unsigned),
        (Id::Tracks, Type::Master),
        (Id::TrackEntry, Type::Master),
        (Id::TrackNumber, Type::Unsigned),
        (Id::TrackUid, Type::Unsigned),
        (Id::TrackType, Type::Unsigned),
        (Id::FlagEnabled, Type::Unsigned),
        (Id::FlagDefault, Type::Unsigned),
        (Id::FlagForced, Type::Unsigned),
        (Id::FlagLacing, Type::Unsigned),
        (Id::DefaultDuration, Type::Unsigned),
        (Id::Name, Type::Utf8),
        (Id::Language, Type::String),
        (Id::CodecId, Type::String),
        (Id::CodecPrivate, Type::Binary),
        (Id::CodecName, Type::Utf8),
        (Id::CodecDelay, Type::Unsigned),
        (Id::SeekPreRoll, Type::Unsigned),
        (Id::Video, Type::Master),
        (Id::FlagInterlaced, Type::Unsigned),
        (Id::StereoMode, Type::Unsigned),
        (Id::AlphaMode, Type::Unsigned),
        (Id::PixelWidth, Type::Unsigned),
        (Id::PixelHeight, Type::Unsigned),
        (Id::PixelCropBottom, Type::Unsigned),
        (Id::PixelCropTop, Type::Unsigned),
        (Id::PixelCropLeft, Type::Unsigned),
        (Id::PixelCropRight, Type::Unsigned),
        (Id::DisplayWidth, Type::Unsigned),
        (Id::DisplayHeight, Type::Unsigned),
        (Id::DisplayUnit, Type::Unsigned),
        (Id::AspectRatioType, Type::Unsigned),
        (Id::FrameRate, Type::Float),
        (Id::Colour, Type::Master),
        (Id::MatrixCoefficients, Type::Unsigned),
        (Id::BitsPerChannel, Type::Unsigned),
        (Id::ChromaSubsamplingHorz, Type::Unsigned),
        (Id::ChromaSubsamplingVert, Type::Unsigned),
        (Id::CbSubsamplingHorz, Type::Unsigned),
        (Id::CbSubsamplingVert, Type::Unsigned),
        (Id::ChromaSitingHorz, Type::Unsigned),
        (Id::ChromaSitingVert, Type::Unsigned),
        (Id::Range, Type::Unsigned),
        (Id::TransferCharacteristics, Type::Unsigned),
        (Id::Primaries, Type::Unsigned),
        (Id::MaxCll, Type::Unsigned),
        (Id::MaxFall, Type::Unsigned),
        (Id::MasteringMetadata, Type::Master),
        (Id::PrimaryRChromaticityX, Type::Float),
        (Id::PrimaryRChromaticityY, Type::Float),
        (Id::PrimaryGChromaticityX, Type::Float),
        (Id::PrimaryGChromaticityY, Type::Float),
        (Id::PrimaryBChromaticityX, Type::Float),
        (Id::PrimaryBChromaticityY, Type::Float),
        (Id::WhitePointChromaticityX, Type::Float),
        (Id::WhitePointChromaticityY, Type::Float),
        (Id::LuminanceMax, Type::Float),
        (Id::LuminanceMin, Type::Float),
        (Id::Projection, Type::Master),
        (Id::ProjectionType, Type::Unsigned),
        (Id::ProjectionPrivate, Type::Binary),
        (Id::ProjectionPoseYaw, Type::Float),
        (Id::ProjectionPosePitch, Type::Float),
        (Id::ProjectionPoseRoll, Type::Float),
        (Id::Audio, Type::Master),
        (Id::SamplingFrequency, Type::Float),
        (Id::OutputSamplingFrequency, Type::Float),
        (Id::Channels, Type::Unsigned),
        (Id::BitDepth, Type::Unsigned),
        (Id::ContentEncodings, Type::Master),
        (Id::ContentEncoding, Type::Master),
        (Id::ContentEncodingOrder, Type::Unsigned),
        (Id::ContentEncodingScope, Type::Unsigned),
        (Id::ContentEncodingType, Type::Unsigned),
        (Id::ContentEncryption, Type::Master),
        (Id::ContentEncAlgo, Type::Unsigned),
        (Id::ContentEncKeyId, Type::Binary),
        (Id::ContentEncAesSettings, Type::Master),
        (Id::AesSettingsCipherMode, Type::Unsigned),
        (Id::Cues, Type::Master),
        (Id::CuePoint, Type::Master),
        (Id::CueTime, Type::Unsigned),
        (Id::CueTrackPositions, Type::Master),
        (Id::CueTrack, Type::Unsigned),
        (Id::CueClusterPosition, Type::Unsigned),
        (Id::CueRelativePosition, Type::Unsigned),
        (Id::CueDuration, Type::Unsigned),
        (Id::CueBlockNumber, Type::Unsigned),
        (Id::Chapters, Type::Master),
        (Id::EditionEntry, Type::Master),
        (Id::ChapterAtom, Type::Master),
        (Id::ChapterUid, Type::Unsigned),
        (Id::ChapterStringUid, Type::Utf8),
        (Id::ChapterTimeStart, Type::Unsigned),
        (Id::ChapterTimeEnd, Type::Unsigned),
        (Id::ChapterDisplay, Type::Master),
        (Id::ChapString, Type::Utf8),
        (Id::ChapLanguage, Type::String),
        (Id::ChapCountry, Type::String),
        (Id::Tags, Type::Master),
        (Id::Tag, Type::Master),
        (Id::Targets, Type::Master),
        (Id::TargetTypeValue, Type::Unsigned),
        (Id::TargetType, Type::String),
        (Id::TagTrackUid, Type::Unsigned),
        (Id::SimpleTag, Type::Master),
        (Id::TagName, Type::Utf8),
        (Id::TagLanguage, Type::String),
        (Id::TagDefault, Type::Unsigned),
        (Id::TagString, Type::Utf8),
        (Id::TagBinary, Type::Binary),
    ]);
}

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

    Ok((
        input,
        Id::from_u32(id).unwrap_or_else(|| {
            eprintln!("Failed to parse ID 0x{:X}. Is it supported by WebM?", id);
            eprintln!("Check here: https://www.webmproject.org/docs/container/");
            Id::Unknown
        }),
    ))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Header {
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
#[serde(untagged)]
pub enum BinaryValue {
    SeekId(Id),
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Body {
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
pub struct Element {
    #[serde(flatten)]
    pub header: Header,
    #[serde(rename = "value")]
    pub body: Body,
}

pub fn parse_element(input: &[u8]) -> IResult<&[u8], Element> {
    let (input, header) = parse_header(input)?;
    let element_type = ID_ELEMENT_TYPE_MAP
        .get(&header.id)
        .expect("All IDs should have an entry in the lookup map");

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
                Id::SeekId => parse_id(&value).map_or_else(
                    |_| {
                        eprintln!("Failed parsing SeekId. Returning hidden binary");
                        BinaryValue::Hidden
                    },
                    |(_, id)| BinaryValue::SeekId(id),
                ),
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

pub fn print_element_trees(elements: &[Element], format: &str) {
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
    fn test_parse_unknown() {
        assert_eq!(
            parse_element(&[0xBF, 0x84, 0xAF, 0x93, 0x97, 0x18]),
            Ok((
                EMPTY,
                Element {
                    header: Header::new(Id::Unknown, 2, 4),
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
