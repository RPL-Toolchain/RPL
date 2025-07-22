//@ revisions: inline regular
//@[inline] compile-flags: -Z inline-mir=true
//@[regular] compile-flags: -Z inline-mir=false

#![allow(non_local_definitions)]
use crate::{rational::Rational64, FrameError::InvalidConversion};
use byte_slice_cast::{AsMutSliceOf, AsSliceOf};
use num_derive::{FromPrimitive, ToPrimitive};
use num_rational as rational;
use std::{any::Any, fmt, ptr::copy_nonoverlapping, slice, sync::Arc};
use thiserror::Error;

// TODO: Change it to provide Droppable/Seekable information or use a separate enum?
/// A list of recognized frame types.
#[derive(Clone, Debug, PartialEq)]
pub enum FrameType {
    /// Intra frame type.
    I,
    /// Inter frame type.
    P,
    /// Bidirectionally predicted frame.
    B,
    /// Skip frame.
    ///
    /// When such frame is encountered, then last frame should be used again
    /// if it is needed.
    SKIP,
    /// Some other frame type.
    OTHER,
}

/// YUV color range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YUVRange {
    /// Pixels in the range [16, 235].
    Limited,
    /// Pixels in the range [0, 255].
    Full,
}

/// All YUV color reprentations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YUVSystem {
    YCbCr(YUVRange),
    YCoCg,
    ICtCp,
}

/// Trichromatic color encoding system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrichromaticEncodingSystem {
    RGB,
    YUV(YUVSystem),
    XYZ,
}

/// All supported color models.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorModel {
    Trichromatic(TrichromaticEncodingSystem),
    CMYK,
    HSV,
    LAB,
}

/// Values adopted from Table 4 of ISO/IEC 23001-8:2013/DCOR1.
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
pub enum ColorPrimaries {
    Reserved0 = 0,
    BT709 = 1,
    Unspecified = 2,
    Reserved = 3,
    BT470M = 4,
    BT470BG = 5,
    ST170M = 6,
    ST240M = 7,
    Film = 8,
    BT2020 = 9,
    ST428 = 10,
    P3DCI = 11,
    P3Display = 12,
    Tech3213 = 22,
}

/// Values adopted from Table 4 of ISO/IEC 23001-8:2013/DCOR1.
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
pub enum TransferCharacteristic {
    Reserved0 = 0,
    BT1886 = 1,
    Unspecified = 2,
    Reserved = 3,
    BT470M = 4,
    BT470BG = 5,
    ST170M = 6,
    ST240M = 7,
    Linear = 8,
    Logarithmic100 = 9,
    Logarithmic316 = 10,
    XVYCC = 11,
    BT1361E = 12,
    SRGB = 13,
    BT2020Ten = 14,
    BT2020Twelve = 15,
    PerceptualQuantizer = 16,
    ST428 = 17,
    HybridLogGamma = 18,
}

/// Values adopted from Table 4 of ISO/IEC 23001-8:2013/DCOR1.
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, ToPrimitive)]
pub enum MatrixCoefficients {
    Identity = 0,
    BT709 = 1,
    Unspecified = 2,
    Reserved = 3,
    BT470M = 4,
    BT470BG = 5,
    ST170M = 6,
    ST240M = 7,
    YCgCo = 8,
    BT2020NonConstantLuminance = 9,
    BT2020ConstantLuminance = 10,
    ST2085 = 11,
    ChromaticityDerivedNonConstantLuminance = 12,
    ChromaticityDerivedConstantLuminance = 13,
    ICtCp = 14,
}

/// Values adopted from Table 4 of ISO/IEC 23001-8:2013/DCOR1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChromaLocation {
    Unspecified = 0,
    Left,
    Center,
    TopLeft,
    Top,
    BottomLeft,
    Bottom,
}

/// Single colorspace component definition.
///
/// Defines how the components of a colorspace are subsampled and
/// where and how they are stored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chromaton {
    /// Horizontal subsampling in power of two
    /// (e.g. `0` = no subsampling, `1` = only every second value is stored).
    pub h_ss: u8,
    /// Vertial subsampling in power of two
    /// (e.g. `0` = no subsampling, `1` = only every second value is stored).
    pub v_ss: u8,
    /// Tells if a component is packed.
    pub packed: bool,
    /// Bit depth of a component.
    pub depth: u8,
    /// Shift for packed components.
    pub shift: u8,
    /// Component offset for byte-packed components.
    pub comp_offs: u8,
    /// The distance to the next packed element in bytes.
    pub next_elem: u8,
}

impl Chromaton {
    /// Calculates the width for a component from general image width.
    pub fn get_width(self, width: usize) -> usize {
        (width + ((1 << self.v_ss) - 1)) >> self.v_ss
    }

    /// Calculates the height for a component from general image height.
    pub fn get_height(self, height: usize) -> usize {
        (height + ((1 << self.h_ss) - 1)) >> self.h_ss
    }
}

/// Image colorspace representation.
///
/// Includes both definitions for each component and some common definitions.
///
/// For example, the format can be paletted, so the components describe
/// the palette storage format, while the actual data is 8-bit palette indices.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Formaton {
    /// Image color model.
    pub model: ColorModel,
    /// Image color primaries.
    pub primaries: ColorPrimaries,
    /// Image transfer characteristic.
    pub xfer: TransferCharacteristic,
    /// Image matrix coefficients.
    pub matrix: MatrixCoefficients,
    /// Image chroma location.
    pub chroma_location: ChromaLocation,

    /// Actual number of components present.
    pub components: u8,
    /// Format definition for each component.
    pub comp_info: [Option<Chromaton>; 5],
    /// Single pixel size for packed formats.
    pub elem_size: u8,
    /// Tells if data is stored as big-endian.
    pub be: bool,
    /// Tells if image has alpha component.
    pub alpha: bool,
    /// Tells if data is paletted.
    pub palette: bool,
}

impl Formaton {
    /// Returns an iterator over the format definition of each component.
    pub fn iter(&self) -> slice::Iter<Option<Chromaton>> {
        self.comp_info.iter()
    }
}

/// Video stream information.
#[derive(Clone, Debug)]
pub struct VideoInfo {
    /// Frame width.
    pub width: usize,
    /// Frame height.
    pub height: usize,
    /// Frame is stored downside up.
    pub flipped: bool,
    /// Frame type.
    pub frame_type: FrameType,
    /// Frame pixel format.
    pub format: Arc<Formaton>,
    /// Declared bits per sample.
    pub bits: u8,
}

impl VideoInfo {
    /// Returns frame width.
    pub fn get_width(&self) -> usize {
        self.width
    }
    /// Returns frame height.
    pub fn get_height(&self) -> usize {
        self.height
    }
}

impl PartialEq for VideoInfo {
    fn eq(&self, info2: &VideoInfo) -> bool {
        self.width == info2.width && self.height == info2.height && self.format == info2.format
    }
}

/// Known audio channel types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelType {
    C,
    L,
    R,
    Cs,
    Ls,
    Rs,
    Lss,
    Rss,
    LFE,
    Lc,
    Rc,
    Lh,
    Rh,
    Ch,
    LFE2,
    Lw,
    Rw,
    Ov,
    Lhs,
    Rhs,
    Chs,
    Ll,
    Rl,
    Cl,
    Lt,
    Rt,
    Lo,
    Ro,
}

/// An ordered sequence of channels.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChannelMap {
    ids: Vec<ChannelType>,
}

/// Audio format definition.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Soniton {
    /// Bits per sample.
    pub bits: u8,
    /// Tells if audio format is big-endian.
    pub be: bool,
    /// Audio samples are packed (e.g. 20-bit audio samples) and not padded.
    pub packed: bool,
    /// Audio data is stored in planar format
    /// (channels in sequence i.e. C1 C1 C1... C2 C2 C2) instead of interleaving
    /// samples (i.e. C1 C2 C1 C2) for different channels.
    pub planar: bool,
    /// Audio data is in floating point format.
    pub float: bool,
    /// Audio data is signed (usually only 8-bit audio is unsigned).
    pub signed: bool,
}

/// Audio stream information contained in a frame.
#[derive(Clone, Debug)]
pub struct AudioInfo {
    /// Number of samples.
    pub samples: usize,
    /// Sample rate.
    pub sample_rate: usize,
    /// Sequence of stream channels.
    pub map: ChannelMap,
    /// Audio sample format.
    pub format: Arc<Soniton>,
    /// Length of one audio block in samples.
    ///
    /// None if not present.
    pub block_len: Option<usize>,
}

impl PartialEq for AudioInfo {
    fn eq(&self, info2: &AudioInfo) -> bool {
        self.sample_rate == info2.sample_rate
            && self.map == info2.map
            && self.format == info2.format
    }
}

/// A list of possible stream information types.
#[derive(Clone, Debug, PartialEq)]
pub enum MediaKind {
    /// Video codec information.
    Video(VideoInfo),
    /// Audio codec information.
    Audio(AudioInfo),
}

/// Frame errors.
#[derive(Debug, Error)]
pub enum FrameError {
    /// Invalid frame index.
    #[error("Invalid Index")]
    InvalidIndex,
    /// Invalid frame conversion.
    #[error("Invalid Conversion")]
    InvalidConversion,
}

mod private {
    use byte_slice_cast::*;

    pub trait Supported: FromByteSlice {}
    impl Supported for u8 {}
    impl Supported for i16 {}
    impl Supported for f32 {}
}

pub trait FrameBuffer: Send + Sync {
    // Get the size of the plane `idx`.
    fn linesize(&self, idx: usize) -> Result<usize, FrameError>;
    // Get the number of planes.
    fn count(&self) -> usize;
    // Get the plane `idx` as a slice.
    fn as_slice_inner(&self, idx: usize) -> Result<&[u8], FrameError>;
    // Get the plane `idx` as a mutable slice.
    fn as_mut_slice_inner(&mut self, idx: usize) -> Result<&mut [u8], FrameError>;
}

impl fmt::Debug for dyn FrameBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FrameBuffer")
    }
}

pub trait FrameBufferConv<T: private::Supported>: FrameBuffer {
    fn as_slice(&self, idx: usize) -> Result<&[T], FrameError> {
        self.as_slice_inner(idx)?
            .as_slice_of::<T>()
            .map_err(|e| InvalidConversion)
    }
    fn as_mut_slice(&mut self, idx: usize) -> Result<&mut [T], FrameError> {
        self.as_mut_slice_inner(idx)?
            .as_mut_slice_of::<T>()
            .map_err(|e| InvalidConversion)
    }
}

impl FrameBufferConv<u8> for dyn FrameBuffer {}
impl FrameBufferConv<i16> for dyn FrameBuffer {}
impl FrameBufferConv<f32> for dyn FrameBuffer {}

/// Timestamp information.
#[derive(Debug, Clone, Default)]
pub struct TimeInfo {
    /// Presentation timestamp.
    pub pts: Option<i64>,
    /// Decode timestamp.
    pub dts: Option<i64>,
    /// Duration (in timebase units).
    pub duration: Option<u64>,
    /// Timebase numerator/denominator.
    pub timebase: Option<Rational64>,
    /// Timebase user private data.
    pub user_private: Option<Arc<dyn Any + Send + Sync>>,
}

#[derive(Debug)]
pub struct Frame {
    pub kind: MediaKind,
    pub buf: Box<dyn FrameBuffer>,
    pub t: TimeInfo,
}

// Copies a plane from `src` to `dst`.
fn copy_plane(
    dst: &mut [u8],
    dst_linesize: usize,
    src: &[u8],
    src_linesize: usize,
    w: usize,
    h: usize,
) {
    let dst_chunks = dst.chunks_mut(dst_linesize);
    let src_chunks = src.chunks(src_linesize);

    for (d, s) in dst_chunks.zip(src_chunks).take(h) {
        unsafe {
            copy_nonoverlapping(s.as_ptr(), d.as_mut_ptr(), w);
        }
    }
}

impl Frame {
    // `src` and `src_linesize` are raw pointers and buffer lengths to copy from.
    // Each `src` is a pointer to the beginning of a plane.
    // Each `src_linesize` is the number of bytes between the start of two consecutive lines in a plane.
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    pub fn copy_from_raw_parts<I, IU>(&mut self, mut src: I, mut src_linesize: IU)
    //~^ERROR: it is unsound to trust pointers from passed-in iterators in a public safe function
    where
        I: Iterator<Item = *const u8>,
        IU: Iterator<Item = usize>,
    {
        if let MediaKind::Video(ref fmt) = self.kind {
            let mut f_iter = fmt.format.iter();
            let width = fmt.width;
            let height = fmt.height;
            for i in 0..self.buf.count() {
                let d_linesize = self.buf.linesize(i).unwrap();
                let s_linesize = src_linesize.next().unwrap();
                let data = self.buf.as_mut_slice(i).unwrap();
                let cc = f_iter.next().unwrap();
                let rr = src.next().unwrap();
                let hb = cc.unwrap().get_height(height);
                // - `rr` is the next item from the iterator `src`.
                //   And it is the beginning of the plane.
                // - `hb` is the next item from the iterator `cc`, which comes from `fmt.format`.
                //   And it is the height of the plane, also equal to the number of lines in the plane.
                // - `s_linesize` is the next item from the iterator `src_linesize`.
                //   And it is the number of bytes between the start of two consecutive lines in a plane.
                let ss = unsafe { slice::from_raw_parts(rr, hb * s_linesize) };
                copy_plane(
                    data,
                    d_linesize,
                    ss,
                    s_linesize,
                    cc.unwrap().get_width(width),
                    hb,
                );
            }
        } else {
            unimplemented!();
        }
    }
}

fn main() {
    panic!("Todo.");
}
