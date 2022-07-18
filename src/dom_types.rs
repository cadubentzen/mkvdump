use crate::{Element, ElementMetadata};

/// Data structures representing parsed DOM objects.
///
/// For more information on each type and member, see the WebM specification for
/// the element that each type/member represents.

/// Metadata for a single frame.
#[derive(Debug, PartialEq)]
pub struct FrameMetadata {
    /// Metadata for the EBML element (\WebMID{Block} or \WebMID{SimpleBlock}) that
    /// contains this frame.
    pub parent_element: ElementMetadata,
    /// Absolute byte position (from the beginning of the byte stream/file) of the
    /// frame start.
    pub position: u64,
    /// Size (in bytes) of the frame.
    pub size: u64,
}

// #[derive(Debug, PartialEq)]
// pub struct BlockMore {
//     pub id: Element<u64>,
//     pub data: Element<Vec<u8>>,
// }

// #[derive(Debug, PartialEq)]
// pub struct BlockAdditions {}

// #[derive(Debug, PartialEq)]
// pub struct TimeSlice {}

// #[derive(Debug, PartialEq)]
// pub struct Slices {}

// #[derive(Debug, PartialEq)]
// pub struct VirtualBlock {}

// #[repr(u8)]
// #[derive(Debug, PartialEq)]
// pub enum Lacing {}

// #[derive(Debug, PartialEq)]
// pub struct Block {}

// #[derive(Debug, PartialEq)]
// pub struct SimpleBlock {}

// #[derive(Debug, PartialEq)]
// pub struct BlockGroup {}

// #[derive(Debug, PartialEq)]
// pub struct Cluster {}

// #[derive(Debug, PartialEq)]
// pub struct Ebml {}

// #[derive(Debug, PartialEq)]
// pub struct Info {}

// #[derive(Debug, PartialEq)]
// pub struct Seek {}

// #[derive(Debug, PartialEq)]
// pub struct Audio {}

// #[derive(Debug, PartialEq)]
// pub struct MasteringMetadata {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum MatrixCoefficients {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum Range {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum TransferCharacteristics {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum Primaries {}

// #[derive(Debug, PartialEq)]
// pub struct Colour {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum ProjectionType {}

// #[derive(Debug, PartialEq)]
// pub struct Projection {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum FlagInterlaced {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum StereoMode {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum DisplayUnit {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum AspectRatioType {}

// #[derive(Debug, PartialEq)]
// pub struct Video {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum AesSettingsCipherMode {}

// #[derive(Debug, PartialEq)]
// pub struct ContentEncAesSettings {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum ContentEncAlgo {}

// #[derive(Debug, PartialEq)]
// pub struct ContentEncryption {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum ContentEncodingType {}

// #[derive(Debug, PartialEq)]
// pub struct ContentEncoding {}

// #[derive(Debug, PartialEq)]
// pub struct ContentEncodings {}

// #[repr(u64)]
// #[derive(Debug, PartialEq)]
// pub enum TrackType {}

// #[derive(Debug, PartialEq)]
// pub struct TrackEntry {}

// #[derive(Debug, PartialEq)]
// pub struct CueTrackPositions {}

// #[derive(Debug, PartialEq)]
// pub struct CuePoint {}

// #[derive(Debug, PartialEq)]
// pub struct ChapterDisplay {}

// #[derive(Debug, PartialEq)]
// pub struct ChapterAtom {}

// #[derive(Debug, PartialEq)]
// pub struct EditionEntry {}

// #[derive(Debug, PartialEq)]
// pub struct SimpleTag {}

// #[derive(Debug, PartialEq)]
// pub struct Targets {}

// #[derive(Debug, PartialEq)]
// pub struct Tag {}
