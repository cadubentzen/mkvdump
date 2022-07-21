use crate::Id;

static EBML_ANCESTORY: [Id; 1] = [Id::Ebml];
static SEEK_ANCESTORY: [Id; 3] = [Id::Segment, Id::SeekHead, Id::Seek];
static INFO_ANCESTORY: [Id; 2] = [Id::Segment, Id::Info];
static BLOCK_MORE_ANCESTORY: [Id; 5] = [
    Id::Segment,
    Id::Cluster,
    Id::BlockGroup,
    Id::BlockAdditions,
    Id::BlockMore,
];
static TIME_SLICE_ANCESTORY: [Id; 5] = [
    Id::Segment,
    Id::Cluster,
    Id::BlockGroup,
    Id::Slices,
    Id::TimeSlice,
];
static VIDEO_ANCESTORY: [Id; 4] = [Id::Segment, Id::Tracks, Id::TrackEntry, Id::Video];
static AUDIO_ANCESTORY: [Id; 4] = [Id::Segment, Id::Tracks, Id::TrackEntry, Id::Audio];
static CONTENT_ENC_AES_SETTINGS_ANCESTORY: [Id; 7] = [
    Id::Segment,
    Id::Tracks,
    Id::TrackEntry,
    Id::ContentEncodings,
    Id::ContentEncoding,
    Id::ContentEncryption,
    Id::ContentEncAesSettings,
];
static CUE_TRACK_POSITIONS_ANCESTORY: [Id; 4] =
    [Id::Segment, Id::Cues, Id::CuePoint, Id::CueTrackPositions];
static CHAPTER_DISPLAY_ANCESTORY: [Id; 5] = [
    Id::Segment,
    Id::Chapters,
    Id::EditionEntry,
    Id::ChapterAtom,
    Id::ChapterDisplay,
];
static TARGETS_ANCESTORY: [Id; 4] = [Id::Segment, Id::Tags, Id::Tag, Id::Targets];
static SIMPLE_TAG_ANCESTORY: [Id; 4] = [Id::Segment, Id::Tags, Id::Tag, Id::SimpleTag];

#[derive(Debug)]
pub struct Ancestory<'a> {
    ancestory: &'a [Id],
}

impl<'a> Ancestory<'a> {
    pub fn by_id(id: Id) -> Option<Self> {
        match id {
            Id::EbmlVersion
            | Id::EbmlReadVersion
            | Id::EbmlMaxIdLength
            | Id::EbmlMaxSizeLength
            | Id::DocType
            | Id::DocTypeVersion
            | Id::DocTypeReadVersion => Some(Self::new(&EBML_ANCESTORY[..1])),
            Id::SeekHead
            | Id::Info
            | Id::Cluster
            | Id::Tracks
            | Id::Cues
            | Id::Chapters
            | Id::Tags => Some(Self::new(&SEEK_ANCESTORY[..1])),
            Id::Seek => Some(Self::new(&SEEK_ANCESTORY[..2])),
            Id::SeekId | Id::SeekPosition => Some(Self::new(&SEEK_ANCESTORY[..3])),
            Id::TimecodeScale
            | Id::Duration
            | Id::DateUtc
            | Id::Title
            | Id::MuxingApp
            | Id::WritingApp => Some(Self::new(&INFO_ANCESTORY[..2])),
            Id::Timecode | Id::PrevSize | Id::SimpleBlock | Id::BlockGroup => {
                Some(Self::new(&BLOCK_MORE_ANCESTORY[..2]))
            }
            Id::Block
            | Id::BlockVirtual
            | Id::BlockAdditions
            | Id::BlockDuration
            | Id::ReferenceBlock
            | Id::DiscardPadding
            | Id::Slices => Some(Self::new(&BLOCK_MORE_ANCESTORY[..3])),
            Id::BlockMore => Some(Self::new(&BLOCK_MORE_ANCESTORY[..4])),
            Id::BlockAddId | Id::BlockAdditional => Some(Self::new(&BLOCK_MORE_ANCESTORY[..5])),
            Id::TimeSlice => Some(Self::new(&TIME_SLICE_ANCESTORY[..4])),
            Id::LaceNumber => Some(Self::new(&TIME_SLICE_ANCESTORY[..5])),
            Id::TrackEntry => Some(Self::new(&VIDEO_ANCESTORY[..2])),
            Id::TrackNumber
            | Id::TrackUid
            | Id::TrackType
            | Id::FlagEnabled
            | Id::FlagDefault
            | Id::FlagForced
            | Id::FlagLacing
            | Id::DefaultDuration
            | Id::Name
            | Id::Language
            | Id::CodecId
            | Id::CodecPrivate
            | Id::CodecName
            | Id::CodecDelay
            | Id::SeekPreRoll
            | Id::Video
            | Id::Audio
            | Id::ContentEncodings => Some(Self::new(&VIDEO_ANCESTORY[..3])),
            Id::FlagInterlaced
            | Id::StereoMode
            | Id::AlphaMode
            | Id::PixelWidth
            | Id::PixelHeight
            | Id::PixelCropBottom
            | Id::PixelCropTop
            | Id::PixelCropLeft
            | Id::PixelCropRight
            | Id::DisplayWidth
            | Id::DisplayHeight
            | Id::DisplayUnit
            | Id::AspectRatioType
            | Id::FrameRate => Some(Self::new(&VIDEO_ANCESTORY[..4])),
            Id::SamplingFrequency | Id::OutputSamplingFrequency | Id::Channels | Id::BitDepth => {
                Some(Self::new(&AUDIO_ANCESTORY[..4]))
            }
            Id::ContentEncoding => Some(Self::new(&CONTENT_ENC_AES_SETTINGS_ANCESTORY[..4])),
            Id::ContentEncodingOrder
            | Id::ContentEncodingScope
            | Id::ContentEncodingType
            | Id::ContentEncryption => Some(Self::new(&CONTENT_ENC_AES_SETTINGS_ANCESTORY[..5])),
            Id::ContentEncAlgo | Id::ContentEncKeyId | Id::ContentEncAesSettings => {
                Some(Self::new(&CONTENT_ENC_AES_SETTINGS_ANCESTORY[..6]))
            }
            Id::AesSettingsCipherMode => Some(Self::new(&CONTENT_ENC_AES_SETTINGS_ANCESTORY[..7])),
            Id::CuePoint => Some(Self::new(&CUE_TRACK_POSITIONS_ANCESTORY[..2])),
            Id::CueTime | Id::CueTrackPositions => {
                Some(Self::new(&CUE_TRACK_POSITIONS_ANCESTORY[..3]))
            }
            Id::CueTrack
            | Id::CueClusterPosition
            | Id::CueRelativePosition
            | Id::CueDuration
            | Id::CueBlockNumber => Some(Self::new(&CUE_TRACK_POSITIONS_ANCESTORY[..4])),
            Id::EditionEntry => Some(Self::new(&CHAPTER_DISPLAY_ANCESTORY[..2])),
            Id::ChapterAtom => Some(Self::new(&CHAPTER_DISPLAY_ANCESTORY[..3])),
            Id::ChapterUid
            | Id::ChapterStringUid
            | Id::ChapterTimeStart
            | Id::ChapterTimeEnd
            | Id::ChapterDisplay => Some(Self::new(&CHAPTER_DISPLAY_ANCESTORY[..4])),
            Id::ChapString | Id::ChapLanguage | Id::ChapCountry => {
                Some(Self::new(&CHAPTER_DISPLAY_ANCESTORY[..5]))
            }
            Id::Tag => Some(Self::new(&TARGETS_ANCESTORY[..2])),
            Id::Targets | Id::SimpleTag => Some(Self::new(&TARGETS_ANCESTORY[..3])),
            Id::TargetTypeValue | Id::TargetType | Id::TagTrackUid => {
                Some(Self::new(&TARGETS_ANCESTORY[..4]))
            }
            Id::TagName | Id::TagLanguage | Id::TagDefault | Id::TagString | Id::TagBinary => {
                Some(Self::new(&SIMPLE_TAG_ANCESTORY[..4]))
            }
            Id::Ebml | Id::Segment => Some(Self::new(&[])),
            _ => None,
        }
    }

    pub fn next(&self) -> Option<Self> {
        self.ancestory.split_first().map(|a| Self::new(a.1))
    }

    pub fn id(&self) -> Option<Id> {
        self.ancestory.get(0).cloned()
    }

    pub const fn is_empty(&self) -> bool {
        self.ancestory.is_empty()
    }

    const fn new(ancestory: &'a [Id]) -> Self {
        Self { ancestory }
    }
}
