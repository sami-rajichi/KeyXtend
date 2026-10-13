//! The RIFF/WAVE chunk walker that kx-ui's build and `cargo xtask sounds` share, and the strict reader for one shipped
//! sound. Every size is checked, so no input panics.

use thiserror::Error;

use super::Format;

/// The RIFF header's id.
pub const RIFF: &[u8] = b"RIFF";
/// The WAVE form's id.
pub const WAVE: &[u8] = b"WAVE";
/// The format chunk's id.
pub const FMT: &[u8] = b"fmt ";
/// The sample chunk's id.
pub const DATA: &[u8] = b"data";
/// Format tag of integer PCM.
pub const PCM: u16 = 1;
/// Bytes of a basic `fmt ` chunk.
pub const FMT_LEN: u16 = 16;
/// Bytes of a chunk id.
pub const ID_LEN: usize = 4;
/// Bytes of a chunk header: its id and length.
pub const CHUNK_HEAD: usize = 8;
/// Where each basic `fmt ` field sits: tag, channels, rate, block size and bits.
const TAG_AT: usize = 0;
const CHANNELS_AT: usize = 2;
const RATE_AT: usize = 4;
const ALIGN_AT: usize = 12;
const BITS_AT: usize = 14;

/// Why a WAV file was refused.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WavError {
    /// The file ends inside a part it announces.
    #[error("the WAV file is truncated")]
    Truncated,
    /// The file is not RIFF/WAVE.
    #[error("not a RIFF/WAVE file")]
    NotWave,
    /// A required chunk is missing.
    #[error("the WAV file has no {:?} chunk", String::from_utf8_lossy(.0))]
    Missing(&'static [u8]),
    /// The file is valid WAV in a layout the reader does not take.
    #[error("unsupported WAV layout: {0}")]
    Unsupported(String),
    /// A header value contradicts the file.
    #[error("malformed WAV file: {0}")]
    Malformed(&'static str),
    /// The sound is longer than the limit named.
    #[error("the sound is longer than {0}")]
    TooLong(String),
}

/// The first `fmt ` and `data` chunks of a WAV file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chunks<'a> {
    /// The `fmt ` chunk's content.
    pub fmt: &'a [u8],
    /// The `data` chunk's content: the samples.
    pub data: &'a [u8],
}

/// The fields of a basic `fmt ` chunk, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fmt {
    /// The format tag, such as `PCM`.
    pub tag: u16,
    /// Channels.
    pub channels: u16,
    /// Samples per second.
    pub rate: u32,
    /// Bytes per frame.
    pub align: u16,
    /// Bits per sample.
    pub bits: u16,
}

impl Fmt {
    /// Reads the basic fields of a `fmt ` chunk's content.
    ///
    /// # Errors
    ///
    /// The chunk is shorter than a basic one.
    pub fn parse(fmt: &[u8]) -> Result<Self, WavError> {
        let short = || WavError::Malformed("the fmt chunk is too short");
        let field = |at| u16_at(fmt, at).ok_or_else(short);
        Ok(Self {
            tag: field(TAG_AT)?,
            channels: field(CHANNELS_AT)?,
            rate: u32_at(fmt, RATE_AT).ok_or_else(short)?,
            align: field(ALIGN_AT)?,
            bits: field(BITS_AT)?,
        })
    }

    /// Refuses a layout other than PCM in `format`'s channels, bits and rate, and a block size that contradicts it.
    fn fits(self, format: &Format) -> Result<(), WavError> {
        let have = (self.tag, self.channels, self.bits, self.rate);
        let want = (PCM, format.channels, format.bits, format.rate);
        if have != want {
            let why = format!("{}; sounds ship as {}", layout(have), layout(want));
            return Err(WavError::Unsupported(why));
        }
        if usize::from(self.align) != format.frame_bytes() {
            return Err(WavError::Malformed("the block size is wrong"));
        }
        Ok(())
    }
}

/// The sample bytes of `bytes`, a WAV file that must be PCM in `format`, in whole frames, from one frame to `max_ms`.
///
/// # Errors
///
/// Any other file, with the reason.
pub fn samples<'a>(bytes: &'a [u8], format: &Format) -> Result<&'a [u8], WavError> {
    let Chunks { fmt, data } = chunks(bytes)?;
    Fmt::parse(fmt)?.fits(format)?;
    let frame = format.frame_bytes();
    if frame == 0 || !data.len().is_multiple_of(frame) {
        return Err(WavError::Malformed("the data ends inside a frame"));
    }
    if data.is_empty() {
        return Err(WavError::Malformed("the data holds no sample"));
    }
    if data.len() / frame > format.max_frames() {
        return Err(WavError::TooLong(format!("{} ms", format.max_ms)));
    }
    Ok(data)
}

/// The first `fmt ` and `data` chunks of the WAV file `bytes`; other chunks are skipped.
///
/// # Errors
///
/// The file is not RIFF/WAVE, ends inside a chunk, or lacks either chunk.
pub fn chunks(bytes: &[u8]) -> Result<Chunks<'_>, WavError> {
    let mut body = riff_body(bytes)?;
    let (mut fmt, mut data) = (None, None);
    while !body.is_empty() {
        let head = body.get(..CHUNK_HEAD).ok_or(WavError::Truncated)?;
        let end = u32_at(head, ID_LEN)
            .and_then(|n| usize::try_from(n).ok())
            .and_then(|n| n.checked_add(CHUNK_HEAD));
        let content = end
            .and_then(|end| body.get(CHUNK_HEAD..end))
            .ok_or(WavError::Truncated)?;
        match &head[..ID_LEN] {
            FMT => fmt = fmt.or(Some(content)),
            DATA => data = data.or(Some(content)),
            _ => {}
        }
        // A chunk of odd length has a pad byte, which some writers leave out at the end.
        let next = CHUNK_HEAD + content.len() + content.len() % 2;
        body = body.get(next..).unwrap_or_default();
    }
    let fmt = fmt.ok_or(WavError::Missing(FMT))?;
    let data = data.ok_or(WavError::Missing(DATA))?;
    Ok(Chunks { fmt, data })
}

/// The RIFF body after the `WAVE` id, checking that the announced size fits the file.
fn riff_body(bytes: &[u8]) -> Result<&[u8], WavError> {
    let start = CHUNK_HEAD + ID_LEN;
    // The ids are checked on whatever bytes exist, so a short file of another kind is named as such.
    let form = bytes
        .get(CHUNK_HEAD..start.min(bytes.len()))
        .unwrap_or_default();
    if !bytes.starts_with(&RIFF[..bytes.len().min(ID_LEN)]) || !WAVE.starts_with(form) {
        return Err(WavError::NotWave);
    }
    let head = bytes.get(..start).ok_or(WavError::Truncated)?;
    let end = u32_at(head, ID_LEN)
        .and_then(|n| usize::try_from(n).ok())
        .and_then(|n| n.checked_add(CHUNK_HEAD))
        .filter(|end| *end >= start);
    end.and_then(|end| bytes.get(start..end))
        .ok_or(WavError::Truncated)
}

/// A layout in words: tag, channels, bits and rate.
fn layout((tag, channels, bits, rate): (u16, u16, u16, u32)) -> String {
    format!("tag {tag}, {channels} channels, {bits} bits at {rate} Hz")
}

/// The little-endian `u16` at `at` in `bytes`, if the bytes are there.
#[must_use]
pub fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let pair = bytes.get(at..at.checked_add(size_of::<u16>())?)?;
    Some(u16::from_le_bytes(pair.try_into().ok()?))
}

/// The little-endian `u32` at `at` in `bytes`, if the bytes are there.
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let quad = bytes.get(at..at.checked_add(size_of::<u32>())?)?;
    Some(u32::from_le_bytes(quad.try_into().ok()?))
}
