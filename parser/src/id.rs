use enum_primitive_derive::Primitive;
use nom::{
    bytes::streaming::take,
    combinator::peek,
    error::{Error, ErrorKind},
    Err, IResult,
};
use num_traits::FromPrimitive;

use crate::parser_utils::count_leading_zero_bits;

/// An EBML ID for a WebM element.
///
/// The enum names correspond to the element names from the Matroska and WebM
/// specifications. See those specifications for further information on each
/// element.
///
// For the WebM spec and element info, see:
// http://www.webmproject.org/docs/container/
// http://www.webmproject.org/docs/webm-encryption/#42-new-matroskawebm-elements
// http://matroska.org/technical/specs/index.html
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Primitive, Copy)]
pub enum Id {
    // The MatroskaID alias links to the WebM and Matroska specifications.
    // The WebMID alias links to the WebM specification.
    // The WebMTable alias produces a table given the following arguments:
    //   Type, Level, Mandatory, Multiple, Recursive, Value range, Default value
    /// \MatroskaID{EBML} element ID.
    /// \WebMTable{Master, 0, Yes, Yes, No, , }
    Ebml = 0x1A45DFA3,
    /// \MatroskaID{EBMLVersion} element ID.
    /// \WebMTable{Unsigned integer, 1, Yes, No, No, , 1}
    EbmlVersion = 0x4286,
    /// \MatroskaID{EBMLReadVersion} element ID.
    /// \WebMTable{Unsigned integer, 1, Yes, No, No, , 1}
    EbmlReadVersion = 0x42F7,
    /// \MatroskaID{EBMLMaxIDLength} element ID.
    /// \WebMTable{Unsigned integer, 1, Yes, No, No, , 4}
    EbmlMaxIdLength = 0x42F2,
    /// \MatroskaID{EBMLMaxSizeLength} element ID.
    /// \WebMTable{Unsigned integer, 1, Yes, No, No, , 8}
    EbmlMaxSizeLength = 0x42F3,
    /// \MatroskaID{DocType} element ID.
    /// \WebMTable{ASCII string, 1, Yes, No, No, , matroska}
    DocType = 0x4282,
    /// \MatroskaID{DocTypeVersion} element ID.
    /// \WebMTable{Unsigned integer, 1, Yes, No, No, , 1}
    DocTypeVersion = 0x4287,
    /// \MatroskaID{DocTypeReadVersion} element ID.
    /// \WebMTable{Unsigned integer, 1, Yes, No, No, , 1}
    DocTypeReadVersion = 0x4285,
    /// \MatroskaID{Void} element ID.
    /// \WebMTable{Binary, g, No, No, No, , }
    Void = 0xEC,
    /// \MatroskaID{Segment} element ID.
    /// \WebMTable{Master, 0, Yes, Yes, No, , }
    Segment = 0x18538067,
    /// \MatroskaID{SeekHead} element ID.
    /// \WebMTable{Master, 1, No, Yes, No, , }
    SeekHead = 0x114D9B74,
    /// \MatroskaID{Seek} element ID.
    /// \WebMTable{Master, 2, Yes, Yes, No, , }
    Seek = 0x4DBB,
    /// \MatroskaID{SeekID} element ID.
    /// \WebMTable{Binary, 3, Yes, No, No, , }
    SeekId = 0x53AB,
    /// \MatroskaID{SeekPosition} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, , 0}
    SeekPosition = 0x53AC,
    /// \MatroskaID{Info} element ID.
    /// \WebMTable{Master, 1, Yes, Yes, No, , }
    Info = 0x1549A966,
    /// \MatroskaID{TimecodeScale} element ID.
    /// \WebMTable{Unsigned integer, 2, Yes, No, No, , 1000000}
    TimecodeScale = 0x2AD7B1,
    /// \MatroskaID{Duration} element ID.
    /// \WebMTable{Float, 2, No, No, No, > 0, 0}
    Duration = 0x4489,
    /// \MatroskaID{DateUTC} element ID.
    /// \WebMTable{Date, 2, No, No, No, , 0}
    DateUtc = 0x4461,
    /// \MatroskaID{Title} element ID.
    /// \WebMTable{UTF-8 string, 2, No, No, No, , }
    Title = 0x7BA9,
    /// \MatroskaID{MuxingApp} element ID.
    /// \WebMTable{UTF-8 string, 2, Yes, No, No, , }
    MuxingApp = 0x4D80,
    /// \MatroskaID{WritingApp} element ID.
    /// \WebMTable{UTF-8 string, 2, Yes, No, No, , }
    WritingApp = 0x5741,
    /// \MatroskaID{Cluster} element ID.
    /// \WebMTable{Master, 1, No, Yes, No, , }
    Cluster = 0x1F43B675,
    /// \MatroskaID{Timecode} element ID.
    /// \WebMTable{Unsigned integer, 2, Yes, No, No, , 0}
    Timecode = 0xE7,
    /// \MatroskaID{PrevSize} element ID.
    /// \WebMTable{Unsigned integer, 2, No, No, No, , 0}
    PrevSize = 0xAB,
    /// \MatroskaID{SimpleBlock} element ID.
    /// \WebMTable{Binary, 2, No, Yes, No, , }
    SimpleBlock = 0xA3,
    /// \MatroskaID{BlockGroup} element ID.
    /// \WebMTable{Master, 2, No, Yes, No, , }
    BlockGroup = 0xA0,
    /// \MatroskaID{Block} element ID.
    /// \WebMTable{Binary, 3, Yes, No, No, , }
    Block = 0xA1,
    /// \MatroskaID{BlockVirtual} (deprecated) element ID.
    /// \WebMTable{Binary, 3, No, No, No, , }
    BlockVirtual = 0xA2,
    /// \MatroskaID{BlockAdditions} element ID.
    /// \WebMTable{Master, 3, No, No, No, , }
    BlockAdditions = 0x75A1,
    /// \MatroskaID{BlockMore} element ID.
    /// \WebMTable{Master, 4, Yes, Yes, No, , }
    BlockMore = 0xA6,
    /// \MatroskaID{BlockAddID} element ID.
    /// \WebMTable{Unsigned integer, 5, Yes, No, No, Not 0, 1}
    BlockAddId = 0xEE,
    /// \MatroskaID{BlockAdditional} element ID.
    /// \WebMTable{Binary, 5, Yes, No, No, , }
    BlockAdditional = 0xA5,
    /// \MatroskaID{BlockDuration} element ID.
    /// \WebMTable{Unsigned integer, 3, No, No, No, , DefaultDuration}
    BlockDuration = 0x9B,
    /// \MatroskaID{ReferenceBlock} element ID.
    /// \WebMTable{Signed integer, 3, No, Yes, No, , 0}
    ReferenceBlock = 0xFB,
    /// \MatroskaID{DiscardPadding} element ID.
    /// \WebMTable{Signed integer, 3, No, No, No, , 0}
    DiscardPadding = 0x75A2,
    /// \MatroskaID{Slices} (deprecated).
    /// \WebMTable{Master, 3, No, No, No, , }
    Slices = 0x8E,
    /// \MatroskaID{TimeSlice} (deprecated) element ID.
    /// \WebMTable{Master, 4, No, Yes, No, , }
    TimeSlice = 0xE8,
    /// \MatroskaID{LaceNumber} (deprecated) element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    LaceNumber = 0xCC,
    /// \MatroskaID{Tracks} element ID.
    /// \WebMTable{Master, 1, No, Yes, No, , }
    Tracks = 0x1654AE6B,
    /// \MatroskaID{TrackEntry} element ID.
    /// \WebMTable{Master, 2, Yes, Yes, No, , }
    TrackEntry = 0xAE,
    /// \MatroskaID{TrackNumber} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, Not 0, 0}
    TrackNumber = 0xD7,
    /// \MatroskaID{TrackUID} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, Not 0, 0}
    TrackUid = 0x73C5,
    /// \MatroskaID{TrackType} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, 1-254, 0}
    TrackType = 0x83,
    /// \MatroskaID{FlagEnabled} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, 0-1, 1}
    FlagEnabled = 0xB9,
    /// \MatroskaID{FlagDefault} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, 0-1, 1}
    FlagDefault = 0x88,
    /// \MatroskaID{FlagForced} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, 0-1, 0}
    FlagForced = 0x55AA,
    /// \MatroskaID{FlagLacing} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, 0-1, 1}
    FlagLacing = 0x9C,
    /// \MatroskaID{DefaultDuration} element ID.
    /// \WebMTable{Unsigned integer, 3, No, No, No, Not 0, 0}
    DefaultDuration = 0x23E383,
    /// \MatroskaID{Name} element ID.
    /// \WebMTable{UTF-8 string, 3, No, No, No, , }
    Name = 0x536E,
    /// \MatroskaID{Language} element ID.
    /// \WebMTable{ASCII string, 3, No, No, No, , eng}
    Language = 0x22B59C,
    /// \MatroskaID{CodecID} element ID.
    /// \WebMTable{ASCII string, 3, Yes, No, No, , }
    CodecId = 0x86,
    /// \MatroskaID{CodecPrivate} element ID.
    /// \WebMTable{Binary, 3, No, No, No, , }
    CodecPrivate = 0x63A2,
    /// \MatroskaID{CodecName} element ID.
    /// \WebMTable{UTF-8 string, 3, No, No, No, , }
    CodecName = 0x258688,
    /// \MatroskaID{CodecDelay} element ID.
    /// \WebMTable{Unsigned integer, 3, No, No, No, , 0}
    CodecDelay = 0x56AA,
    /// \MatroskaID{SeekPreRoll} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, , 0}
    SeekPreRoll = 0x56BB,
    /// \MatroskaID{Video} element ID.
    /// \WebMTable{Master, 3, No, No, No, , }
    Video = 0xE0,
    /// \MatroskaID{FlagInterlaced} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, 0-1, 0}
    FlagInterlaced = 0x9A,
    /// \MatroskaID{StereoMode} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    StereoMode = 0x53B8,
    /// \MatroskaID{AlphaMode} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    AlphaMode = 0x53C0,
    /// \MatroskaID{PixelWidth} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, Not 0, 0}
    PixelWidth = 0xB0,
    /// \MatroskaID{PixelHeight} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, Not 0, 0}
    PixelHeight = 0xBA,
    /// \MatroskaID{PixelCropBottom} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    PixelCropBottom = 0x54AA,
    /// \MatroskaID{PixelCropTop} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    PixelCropTop = 0x54BB,
    /// \MatroskaID{PixelCropLeft} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    PixelCropLeft = 0x54CC,
    /// \MatroskaID{PixelCropRight} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    PixelCropRight = 0x54DD,
    /// \MatroskaID{DisplayWidth} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, Not 0, PixelWidth}
    DisplayWidth = 0x54B0,
    /// \MatroskaID{DisplayHeight} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, Not 0, PixelHeight}
    DisplayHeight = 0x54BA,
    /// \MatroskaID{DisplayUnit} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    DisplayUnit = 0x54B2,
    /// \MatroskaID{AspectRatioType} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    AspectRatioType = 0x54B3,
    /// \MatroskaID{FrameRate} (deprecated) element ID.
    /// \WebMTable{Float, 4, No, No, No, > 0, 0}
    FrameRate = 0x2383E3,
    /// \MatroskaID{Colour} element ID.
    /// \WebMTable{Master, 4, No, No, No, , }
    Colour = 0x55B0,
    /// \MatroskaID{MatrixCoefficients} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 2}
    MatrixCoefficients = 0x55B1,
    /// \MatroskaID{BitsPerChannel} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    BitsPerChannel = 0x55B2,
    /// \MatroskaID{ChromaSubsamplingHorz} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    ChromaSubsamplingHorz = 0x55B3,
    /// \MatroskaID{ChromaSubsamplingVert} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    ChromaSubsamplingVert = 0x55B4,
    /// \MatroskaID{CbSubsamplingHorz} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    CbSubsamplingHorz = 0x55B5,
    /// \MatroskaID{CbSubsamplingVert} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    CbSubsamplingVert = 0x55B6,
    /// \MatroskaID{ChromaSitingHorz} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    ChromaSitingHorz = 0x55B7,
    /// \MatroskaID{ChromaSitingVert} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    ChromaSitingVert = 0x55B8,
    /// \MatroskaID{Range} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    Range = 0x55B9,
    /// \MatroskaID{TransferCharacteristics} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 2}
    TransferCharacteristics = 0x55BA,
    /// \MatroskaID{Primaries} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 2}
    Primaries = 0x55BB,
    /// \MatroskaID{MaxCLL} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    MaxCll = 0x55BC,
    /// \MatroskaID{MaxFALL} element ID.
    /// \WebMTable{Unsigned integer, 5, No, No, No, , 0}
    MaxFall = 0x55BD,
    /// \MatroskaID{MasteringMetadata} element ID.
    /// \WebMTable{Master, 5, No, No, No, , }
    MasteringMetadata = 0x55D0,
    /// \MatroskaID{PrimaryRChromaticityX} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    PrimaryRChromaticityX = 0x55D1,
    /// \MatroskaID{PrimaryRChromaticityY} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    PrimaryRChromaticityY = 0x55D2,
    /// \MatroskaID{PrimaryGChromaticityX} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    PrimaryGChromaticityX = 0x55D3,
    /// \MatroskaID{PrimaryGChromaticityY} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    PrimaryGChromaticityY = 0x55D4,
    /// \MatroskaID{PrimaryBChromaticityX} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    PrimaryBChromaticityX = 0x55D5,
    /// \MatroskaID{PrimaryBChromaticityY} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    PrimaryBChromaticityY = 0x55D6,
    /// \MatroskaID{WhitePointChromaticityX} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    WhitePointChromaticityX = 0x55D7,
    /// \MatroskaID{WhitePointChromaticityY} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-1, 0}
    WhitePointChromaticityY = 0x55D8,
    /// \MatroskaID{LuminanceMax} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-9999.99, 0}
    LuminanceMax = 0x55D9,
    /// \MatroskaID{LuminanceMin} element ID.
    /// \WebMTable{Float, 6, No, No, No, 0-999.9999, 0}
    LuminanceMin = 0x55DA,
    /// \WebMID{Projection} element ID.
    /// \WebMTable{Master, 5, No, No, No, , }
    Projection = 0x7670,
    /// \WebMID{ProjectionType} element ID.
    /// \WebMTable{Unsigned integer, 6, Yes, No, No, , 0}
    ProjectionType = 0x7671,
    /// \WebMID{ProjectionPrivate} element ID.
    /// \WebMTable{Binary, 6, No, No, No, , }
    ProjectionPrivate = 0x7672,
    /// \WebMID{ProjectionPoseYaw} element ID.
    /// \WebMTable{Float, 6, Yes, No, No, , 0}
    ProjectionPoseYaw = 0x7673,
    /// \WebMID{ProjectionPosePitch} element ID.
    /// \WebMTable{Float, 6, Yes, No, No, , 0}
    ProjectionPosePitch = 0x7674,
    /// \WebMID{ProjectionPoseRoll} element ID.
    /// \WebMTable{Float, 6, Yes, No, No, , 0}
    ProjectionPoseRoll = 0x7675,
    /// \MatroskaID{Audio} element ID.
    /// \WebMTable{Master, 3, No, No, No, , }
    Audio = 0xE1,
    /// \MatroskaID{SamplingFrequency} element ID.
    /// \WebMTable{Float, 4, Yes, No, No, > 0, 8000}
    SamplingFrequency = 0xB5,
    /// \MatroskaID{OutputSamplingFrequency} element ID.
    /// \WebMTable{Float, 4, No, No, No, > 0, SamplingFrequency}
    OutputSamplingFrequency = 0x78B5,
    /// \MatroskaID{Channels} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, Not 0, 1}
    Channels = 0x9F,
    /// \MatroskaID{BitDepth} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, Not 0, 0}
    BitDepth = 0x6264,
    /// \MatroskaID{ContentEncodings} element ID.
    /// \WebMTable{Master, 3, No, No, No, , }
    ContentEncodings = 0x6D80,
    /// \MatroskaID{ContentEncoding} element ID.
    /// \WebMTable{Master, 4, Yes, Yes, No, , }
    ContentEncoding = 0x6240,
    /// \MatroskaID{ContentEncodingOrder} element ID.
    /// \WebMTable{Unsigned integer, 5, Yes, No, No, , 0}
    ContentEncodingOrder = 0x5031,
    /// \MatroskaID{ContentEncodingScope} element ID.
    /// \WebMTable{Unsigned integer, 5, Yes, No, No, Not 0, 1}
    ContentEncodingScope = 0x5032,
    /// \MatroskaID{ContentEncodingType} element ID.
    /// \WebMTable{Unsigned integer, 5, Yes, No, No, , 0}
    ContentEncodingType = 0x5033,
    /// \MatroskaID{ContentEncryption} element ID.
    /// \WebMTable{Master, 5, No, No, No, , }
    ContentEncryption = 0x5035,
    /// \MatroskaID{ContentEncAlgo} element ID.
    /// \WebMTable{Unsigned integer, 6, No, No, No, , 0}
    ContentEncAlgo = 0x47E1,
    /// \MatroskaID{ContentEncKeyID} element ID.
    /// \WebMTable{Binary, 6, No, No, No, , }
    ContentEncKeyId = 0x47E2,
    /// \WebMID{ContentEncAESSettings} element ID.
    /// \WebMTable{Master, 6, No, No, No, , }
    ContentEncAesSettings = 0x47E7,
    /// \WebMID{AESSettingsCipherMode} element ID.
    /// \WebMTable{Unsigned integer, 7, Yes, No, No, 1, 1}
    AesSettingsCipherMode = 0x47E8,
    /// \MatroskaID{Cues} element ID.
    /// \WebMTable{Master, 1, No, No, No, , }
    Cues = 0x1C53BB6B,
    /// \MatroskaID{CuePoint} element ID.
    /// \WebMTable{Master, 2, Yes, Yes, No, , }
    CuePoint = 0xBB,
    /// \MatroskaID{CueTime} element ID.
    /// \WebMTable{Unsigned integer, 3, Yes, No, No, , 0}
    CueTime = 0xB3,
    /// \MatroskaID{CueTrackPositions} element ID.
    /// \WebMTable{Master, 3, Yes, Yes, No, , }
    CueTrackPositions = 0xB7,
    /// \MatroskaID{CueTrack} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, Not 0, 0}
    CueTrack = 0xF7,
    /// \MatroskaID{CueClusterPosition} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, , 0}
    CueClusterPosition = 0xF1,
    /// \MatroskaID{CueRelativePosition} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    CueRelativePosition = 0xF0,
    /// \MatroskaID{CueDuration} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    CueDuration = 0xB2,
    /// \MatroskaID{CueBlockNumber} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, Not 0, 1}
    CueBlockNumber = 0x5378,
    /// \MatroskaID{Chapters} element ID.
    /// \WebMTable{Master, 1, No, No, No, , }
    Chapters = 0x1043A770,
    /// \MatroskaID{EditionEntry} element ID.
    /// \WebMTable{Master, 2, Yes, Yes, No, , }
    EditionEntry = 0x45B9,
    /// \MatroskaID{ChapterAtom} element ID.
    /// \WebMTable{Master, 3, Yes, Yes, Yes, , }
    ChapterAtom = 0xB6,
    /// \MatroskaID{ChapterUID} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, Not 0, 0}
    ChapterUid = 0x73C4,
    /// \MatroskaID{ChapterStringUID} element ID.
    /// \WebMTable{UTF-8 string, 4, No, No, No, , }
    ChapterStringUid = 0x5654,
    /// \MatroskaID{ChapterTimeStart} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, , 0}
    ChapterTimeStart = 0x91,
    /// \MatroskaID{ChapterTimeEnd} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 0}
    ChapterTimeEnd = 0x92,
    /// \MatroskaID{ChapterDisplay} element ID.
    /// \WebMTable{Master, 4, No, Yes, No, , }
    ChapterDisplay = 0x80,
    /// \MatroskaID{ChapString} element ID.
    /// \WebMTable{UTF-8 string, 5, Yes, No, No, , }
    ChapString = 0x85,
    /// \MatroskaID{ChapLanguage} element ID.
    /// \WebMTable{ASCII string, 5, Yes, Yes, No, , eng}
    ChapLanguage = 0x437C,
    /// \MatroskaID{ChapCountry} element ID.
    /// \WebMTable{ASCII string, 5, No, Yes, No, , }
    ChapCountry = 0x437E,
    /// \MatroskaID{Tags} element ID.
    /// \WebMTable{Master, 1, No, Yes, No, , }
    Tags = 0x1254C367,
    /// \MatroskaID{Tag} element ID.
    /// \WebMTable{Master, 2, Yes, Yes, No, , }
    Tag = 0x7373,
    /// \MatroskaID{Targets} element ID.
    /// \WebMTable{Master, 3, Yes, No, No, , }
    Targets = 0x63C0,
    /// \MatroskaID{TargetTypeValue} element ID.
    /// \WebMTable{Unsigned integer, 4, No, No, No, , 50}
    TargetTypeValue = 0x68CA,
    /// \MatroskaID{TargetType} element ID.
    /// \WebMTable{ASCII string, 4, No, No, No, , }
    TargetType = 0x63CA,
    /// \MatroskaID{TagTrackUID} element ID.
    /// \WebMTable{Unsigned integer, 4, No, Yes, No, , 0}
    TagTrackUid = 0x63C5,
    /// \MatroskaID{SimpleTag} element ID.
    /// \WebMTable{Master, 3, Yes, Yes, Yes, , }
    SimpleTag = 0x67C8,
    /// \MatroskaID{TagName} element ID.
    /// \WebMTable{UTF-8 string, 4, Yes, No, No, , }
    TagName = 0x45A3,
    /// \MatroskaID{TagLanguage} element ID.
    /// \WebMTable{ASCII string, 4, Yes, No, No, , und}
    TagLanguage = 0x447A,
    /// \MatroskaID{TagDefault} element ID.
    /// \WebMTable{Unsigned integer, 4, Yes, No, No, 0-1, 1}
    TagDefault = 0x4484,
    /// \MatroskaID{TagString} element ID.
    /// \WebMTable{UTF-8 string, 4, No, No, No, , }
    TagString = 0x4487,
    /// \MatroskaID{TagBinary} element ID.
    /// \WebMTable{Binary, 4, No, No, No, , }
    TagBinary = 0x4485,
}

pub fn parse_id(input: &[u8]) -> IResult<&[u8], Id> {
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

    if let Some(id) = Id::from_u32(id) {
        Ok((input, id))
    } else {
        Err(Err::Failure(Error::new(input, ErrorKind::Alt)))
    }
}

#[cfg(test)]
mod tests {
    use nom::Needed;

    use super::*;

    #[test]
    fn test_count_leading_zero_bits() {
        assert_eq!(count_leading_zero_bits(0b10000000), 0);
        assert_eq!(count_leading_zero_bits(0b01000000), 1);
        assert_eq!(count_leading_zero_bits(0b00000001), 7);
        assert_eq!(count_leading_zero_bits(0b00000000), 8);
    }

    #[test]
    fn test_parse_id() {
        const EMPTY: &[u8] = &[];
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
}
