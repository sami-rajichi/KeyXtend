//! A tiny WAV writer (16-bit PCM, mono) and a reader for 8, 16 and 24-bit PCM or 32-bit float, mono or stereo.

use std::fmt;

/// Bits per sample the writer makes.
pub(crate) const BITS: u16 = 16;
/// Channels the writer makes.
pub(crate) const CHANNELS: u16 = 1;

/// The ids of the RIFF header, its WAVE form, and the two chunks the reader needs.
const RIFF: &[u8] = b"RIFF";
const WAVE: &[u8] = b"WAVE";
const FMT: &[u8] = b"fmt ";
const DATA: &[u8] = b"data";
/// Format tag of integer PCM.
const PCM: u16 = 1;
/// Format tag of IEEE float.
const FLOAT: u16 = 3;
/// Format tag of the extensible header, whose sub-format names PCM or float.
const EXTENSIBLE: u16 = 0xFFFE;
/// Bytes of a basic `fmt ` chunk.
const FMT_LEN: u16 = 16;
/// Where each `fmt ` field sits: tag, channels, rate, block size, bits and the extensible sub-format.
const TAG_AT: usize = 0;
const CHANNELS_AT: usize = 2;
const RATE_AT: usize = 4;
const ALIGN_AT: usize = 12;
const BITS_AT: usize = 14;
const SUB_FORMAT_AT: usize = 24;
/// Bytes of a chunk id, and of a chunk header (id and length).
const ID_LEN: usize = 4;
const CHUNK_HEAD: usize = 8;
/// Bits in a byte.
const BYTE: u16 = 8;
/// Bits of the integer samples the reader takes: unsigned 8-bit, and signed 16 and 24-bit.
const UNSIGNED_BITS: u16 = 8;
const SIGNED_BITS: [u16; 2] = [16, 24];
/// Bits of the float samples the reader takes.
const FLOAT_BITS: u16 = 32;
/// The most channels the reader mixes: stereo.
const MAX_CHANNELS: u16 = 2;
/// Bytes of the word every sample is widened to.
const WORD: usize = 4;
/// The top bit of a byte, which turns unsigned 8-bit samples into signed ones.
const SIGN_BIT: u8 = 0x80;

/// A recording mixed to mono, with samples from -1 to 1.
#[derive(Debug)]
pub(crate) struct Audio {
    /// Samples per second.
    pub(crate) rate: u32,
    /// The mono samples.
    pub(crate) samples: Vec<f64>,
}

/// Why a WAV file was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WavError {
    /// The file ends inside a part it announces.
    Truncated,
    /// The file is not RIFF/WAVE.
    NotWave,
    /// A required chunk is missing.
    Missing(&'static [u8]),
    /// The file is valid WAV in a layout the reader does not take.
    Unsupported(String),
    /// A header value contradicts the file.
    Malformed(&'static str),
    /// The sound is too long for a WAV file.
    TooLong,
}

impl fmt::Display for WavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => write!(f, "the WAV file is truncated"),
            Self::NotWave => write!(f, "not a RIFF/WAVE file"),
            Self::Missing(id) => {
                let id = String::from_utf8_lossy(id);
                write!(f, "the WAV file has no {id:?} chunk")
            }
            Self::Unsupported(what) => write!(f, "unsupported WAV layout: {what}"),
            Self::Malformed(what) => write!(f, "malformed WAV file: {what}"),
            Self::TooLong => write!(f, "the sound is too long for a WAV file"),
        }
    }
}

/// A 16-bit mono PCM WAV file holding `samples` at `rate`.
pub(crate) fn write(rate: u32, samples: &[i16]) -> Result<Vec<u8>, WavError> {
    let align = CHANNELS * BITS / BYTE;
    let head = ID_LEN + 2 * CHUNK_HEAD + usize::from(FMT_LEN);
    let data_len = u32::try_from(samples.len())
        .ok()
        .and_then(|n| n.checked_mul(u32::from(align)));
    let riff_len = data_len.and_then(|n| n.checked_add(u32::try_from(head).ok()?));
    let byte_rate = rate.checked_mul(u32::from(align));
    let (Some(data_len), Some(riff_len), Some(byte_rate)) = (data_len, riff_len, byte_rate) else {
        return Err(WavError::TooLong);
    };
    let mut out = Vec::with_capacity(CHUNK_HEAD + head + samples.len() * usize::from(align));
    out.extend_from_slice(RIFF);
    out.extend(riff_len.to_le_bytes());
    out.extend_from_slice(WAVE);
    out.extend_from_slice(FMT);
    out.extend(u32::from(FMT_LEN).to_le_bytes());
    out.extend(PCM.to_le_bytes());
    out.extend(CHANNELS.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend(byte_rate.to_le_bytes());
    out.extend(align.to_le_bytes());
    out.extend(BITS.to_le_bytes());
    out.extend_from_slice(DATA);
    out.extend(data_len.to_le_bytes());
    for sample in samples {
        out.extend(sample.to_le_bytes());
    }
    Ok(out)
}

/// Reads a WAV file and mixes it to mono.
pub(crate) fn read(bytes: &[u8]) -> Result<Audio, WavError> {
    let (fmt, data) = chunks(riff_body(bytes)?)?;
    let spec = Spec::parse(fmt)?;
    Ok(Audio {
        rate: spec.rate,
        samples: decode(&spec, data)?,
    })
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

/// The first `fmt ` and `data` chunks of a RIFF body; other chunks are skipped.
fn chunks(mut body: &[u8]) -> Result<(&[u8], &[u8]), WavError> {
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
    Ok((fmt, data.ok_or(WavError::Missing(DATA))?))
}

/// How samples are stored.
#[derive(Debug, Clone, Copy)]
enum Kind {
    /// Unsigned 8-bit integers.
    Unsigned,
    /// Signed little-endian integers.
    Signed,
    /// IEEE 32-bit floats.
    Float,
}

/// The layout a `fmt ` chunk describes.
#[derive(Debug)]
struct Spec {
    /// Samples per second.
    rate: u32,
    /// Channels.
    channels: u16,
    /// Bytes per sample.
    width: usize,
    /// How samples are stored.
    kind: Kind,
}

impl Spec {
    /// Reads and checks a `fmt ` chunk.
    fn parse(fmt: &[u8]) -> Result<Self, WavError> {
        let short = || WavError::Malformed("the fmt chunk is too short");
        let field = |at| u16_at(fmt, at).ok_or_else(short);
        let mut tag = field(TAG_AT)?;
        if tag == EXTENSIBLE {
            tag = field(SUB_FORMAT_AT)?;
        }
        let (channels, bits, align) = (field(CHANNELS_AT)?, field(BITS_AT)?, field(ALIGN_AT)?);
        let rate = u32_at(fmt, RATE_AT).ok_or_else(short)?;
        let kind = match (tag, bits) {
            (PCM, UNSIGNED_BITS) => Kind::Unsigned,
            (PCM, b) if SIGNED_BITS.contains(&b) => Kind::Signed,
            (FLOAT, FLOAT_BITS) => Kind::Float,
            _ => {
                return Err(WavError::Unsupported(format!(
                    "format tag {tag} with {bits} bits"
                )));
            }
        };
        if channels == 0 || channels > MAX_CHANNELS {
            return Err(WavError::Unsupported(format!("{channels} channels")));
        }
        let width = bits / BYTE;
        if rate == 0 || u32::from(align) != u32::from(channels) * u32::from(width) {
            return Err(WavError::Malformed("the rate or the block size is wrong"));
        }
        let width = usize::from(width);
        Ok(Self {
            rate,
            channels,
            width,
            kind,
        })
    }
}

/// The samples of `data`, each frame mixed to mono.
fn decode(spec: &Spec, data: &[u8]) -> Result<Vec<f64>, WavError> {
    let frame = spec.width * usize::from(spec.channels);
    if !data.len().is_multiple_of(frame) {
        return Err(WavError::Malformed("the data ends inside a frame"));
    }
    let channels = f64::from(spec.channels);
    let mix = |f: &[u8]| {
        let sum: f64 = f
            .chunks_exact(spec.width)
            .map(|s| value(spec.kind, s))
            .sum();
        Some(sum / channels).filter(|x| x.is_finite())
    };
    data.chunks_exact(frame)
        .map(|f| mix(f).ok_or(WavError::Malformed("a sample is not a number")))
        .collect()
}

/// One sample, from -1 to 1; `bytes` holds one to four bytes, as `Spec::parse` allows.
fn value(kind: Kind, bytes: &[u8]) -> f64 {
    // Integer bytes go to the top of the word, so every width scales by the same full range.
    let mut word = [0u8; WORD];
    word[WORD - bytes.len()..].copy_from_slice(bytes);
    let full = -f64::from(i32::MIN);
    match kind {
        Kind::Float => f64::from(f32::from_le_bytes(word)),
        Kind::Signed => f64::from(i32::from_le_bytes(word)) / full,
        Kind::Unsigned => {
            word[WORD - 1] ^= SIGN_BIT;
            f64::from(i32::from_le_bytes(word)) / full
        }
    }
}

/// The little-endian `u16` at `at`, if the bytes are there.
fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let pair = bytes.get(at..at.checked_add(size_of::<u16>())?)?;
    Some(u16::from_le_bytes(pair.try_into().ok()?))
}

/// The little-endian `u32` at `at`, if the bytes are there.
fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let quad = bytes.get(at..at.checked_add(size_of::<u32>())?)?;
    Some(u32::from_le_bytes(quad.try_into().ok()?))
}

#[cfg(test)]
mod tests;
