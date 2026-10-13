//! A tiny WAV writer (16-bit PCM, mono) and a reader for 8, 16 and 24-bit PCM or 32-bit float, mono or stereo. The
//! RIFF ids, the chunk walker and the errors are kx-theme's, which the app's build reads the shipped sounds with.

pub(crate) use kx_theme::sounds::wav::WavError;
use kx_theme::sounds::wav::{
    CHUNK_HEAD, Chunks, DATA, FMT, FMT_LEN, Fmt, ID_LEN, PCM, RIFF, WAVE, chunks, u16_at,
};

/// Bits per sample the writer makes.
pub(crate) const BITS: u16 = 16;
/// Channels the writer makes.
pub(crate) const CHANNELS: u16 = 1;
/// The limit a write too long for a WAV file names.
const WAV_LIMIT: &str = "a WAV file holds";

/// Format tag of IEEE float.
const FLOAT: u16 = 3;
/// Format tag of the extensible header, whose sub-format names PCM or float.
const EXTENSIBLE: u16 = 0xFFFE;
/// Where the extensible header's sub-format sits in the `fmt ` chunk.
const SUB_FORMAT_AT: usize = 24;
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
    /// The file's channels, before the mix to mono.
    pub(crate) channels: u16,
    /// The file's bits per sample.
    pub(crate) bits: u16,
    /// The mono samples.
    pub(crate) samples: Vec<f64>,
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
        return Err(WavError::TooLong(WAV_LIMIT.into()));
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
    let Chunks { fmt, data } = chunks(bytes)?;
    let spec = Spec::parse(fmt)?;
    Ok(Audio {
        rate: spec.rate,
        channels: spec.channels,
        bits: spec.bits,
        samples: decode(&spec, data)?,
    })
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
    /// Bits per sample.
    bits: u16,
    /// Bytes per sample.
    width: usize,
    /// How samples are stored.
    kind: Kind,
}

impl Spec {
    /// Reads and checks a `fmt ` chunk.
    fn parse(fmt: &[u8]) -> Result<Self, WavError> {
        let Fmt {
            mut tag,
            channels,
            rate,
            align,
            bits,
        } = Fmt::parse(fmt)?;
        if tag == EXTENSIBLE {
            let none = WavError::Malformed("the extensible fmt chunk has no sub-format");
            tag = u16_at(fmt, SUB_FORMAT_AT).ok_or(none)?;
        }
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
            bits,
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

#[cfg(test)]
mod tests;
