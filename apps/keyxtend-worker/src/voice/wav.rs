//! WAV files the voice job reads: the card check's shipped clips and, in gate builds, the sound played in place of the
//! microphone. It takes 16-bit PCM and 32-bit float within `RATES`, `MAX_CHANNELS` and `MAX_WAV_BYTES` into wiped
//! buffers, as a clip may be someone's voice, and anything else is `BadFile`.

use super::mic::Format;
use super::wipe::Buf;
use crate::riff::{
    BYTE_BITS, CHUNK_HEAD, DATA_ID, FIRST_CHUNK, FMT_ID, FMT_LEAST, PCM, PCM_BITS, RIFF_ID, WAVE_ID,
};
use kx_ipc::VoiceFault;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::ops::RangeInclusive;
use std::path::Path;
use zeroize::Zeroizing;

/// The largest WAV file read: about 8 minutes of 16 kHz mono 16-bit sound, or 4 of float.
pub const MAX_WAV_BYTES: u64 = 16 << 20;
/// The rates a file may have, from telephone sound to the highest studio rate, so a resample stays bounded.
const RATES: RangeInclusive<u32> = 8_000..=384_000;
/// The most channels a file may have, as a microphone array gives.
const MAX_CHANNELS: u16 = 8;

/// Where `WAVE` sits.
const WAVE_AT: usize = 8;
/// Bytes of a chunk id.
const ID_LEN: usize = 4;
/// Offsets in the `fmt ` chunk body: the format tag, channels, rate and bits per sample.
const TAG_AT: usize = 0;
const CHANNELS_AT: usize = 2;
const RATE_AT: usize = 4;
const BITS_AT: usize = 14;
/// The IEEE float format tag, and the width it is read in.
const FLOAT: u16 = 3;
const FLOAT_BITS: u16 = 32;

/// A file's sound: its format, and its samples as floats in -1 to 1, interleaved.
pub struct Sound {
    /// Its rate and channels.
    pub format: Format,
    /// Interleaved samples.
    pub samples: Buf<f32>,
}

/// How the samples are stored.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Coding {
    Pcm16,
    Float32,
}

impl Coding {
    /// Bytes in one sample.
    fn width(self) -> usize {
        usize::from(
            match self {
                Self::Pcm16 => PCM_BITS,
                Self::Float32 => FLOAT_BITS,
            } / BYTE_BITS,
        )
    }
}

/// Reads the WAV file at `path` into room taken at once for its size, so no growth leaves an unwiped copy.
///
/// # Errors
/// `NoFile` when it is missing, `BadFile` when it is too big or not a WAV file this reader takes, `Memory` when its
/// buffers cannot be had.
pub fn read(path: &Path) -> Result<Sound, VoiceFault> {
    let file = File::open(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => VoiceFault::NoFile,
        _ => VoiceFault::BadFile,
    })?;
    let size = file.metadata().map_err(|_| VoiceFault::BadFile)?.len();
    if size > MAX_WAV_BYTES {
        return Err(VoiceFault::BadFile);
    }
    let mut bytes = Zeroizing::new(Vec::new());
    let room = usize::try_from(size).map_err(|_| VoiceFault::BadFile)?;
    bytes
        .try_reserve_exact(room)
        .map_err(|_| VoiceFault::Memory)?;
    file.take(MAX_WAV_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| VoiceFault::BadFile)?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_WAV_BYTES {
        return Err(VoiceFault::BadFile);
    }
    parse(&bytes)
}

/// The sound in `bytes`, a whole WAV file.
///
/// # Errors
/// As `read`, without `NoFile`.
pub fn parse(bytes: &[u8]) -> Result<Sound, VoiceFault> {
    let wave = bytes.get(WAVE_AT..WAVE_AT + ID_LEN);
    if bytes.get(..ID_LEN) != Some(RIFF_ID) || wave != Some(WAVE_ID) {
        return Err(VoiceFault::BadFile);
    }
    let (mut at, mut fmt, mut data) = (FIRST_CHUNK, None, None);
    while let Some((id, body)) = chunk(bytes, at) {
        match id {
            FMT_ID => fmt = Some(format(body)?),
            DATA_ID if data.is_none() => data = Some(body),
            _ => {}
        }
        let len = body.len();
        at = at + CHUNK_HEAD + len + (len & 1);
    }
    let ((format, coding), data) = fmt.zip(data).ok_or(VoiceFault::BadFile)?;
    Ok(Sound {
        format,
        samples: decode(data, coding, usize::from(format.channels))?,
    })
}

/// The chunk at `at`: its id and its body, cut to the bytes held when it declares more.
fn chunk(bytes: &[u8], at: usize) -> Option<(&[u8], &[u8])> {
    let id = bytes.get(at..at.checked_add(ID_LEN)?)?;
    let size = u32_at(bytes, at + ID_LEN)?;
    let body = at + CHUNK_HEAD;
    let end = body
        .saturating_add(usize::try_from(size).ok()?)
        .min(bytes.len());
    Some((id, bytes.get(body..end)?))
}

/// The format a `fmt ` body names, when this reader takes it.
fn format(body: &[u8]) -> Result<(Format, Coding), VoiceFault> {
    if body.len() < FMT_LEAST {
        return Err(VoiceFault::BadFile);
    }
    let field = |at| u16_at(body, at).ok_or(VoiceFault::BadFile);
    let coding = match (field(TAG_AT)?, field(BITS_AT)?) {
        (PCM, PCM_BITS) => Coding::Pcm16,
        (FLOAT, FLOAT_BITS) => Coding::Float32,
        _ => return Err(VoiceFault::BadFile),
    };
    let channels = field(CHANNELS_AT)?;
    let rate = u32_at(body, RATE_AT).ok_or(VoiceFault::BadFile)?;
    if !(1..=MAX_CHANNELS).contains(&channels) || !RATES.contains(&rate) {
        return Err(VoiceFault::BadFile);
    }
    Ok((Format { rate, channels }, coding))
}

/// The whole frames of `data` as floats; a cut-off last frame is dropped.
fn decode(data: &[u8], coding: Coding, channels: usize) -> Result<Buf<f32>, VoiceFault> {
    let width = coding.width();
    let whole = data.len() / (width * channels) * (width * channels);
    let samples = data[..whole].chunks_exact(width);
    Buf::collect(samples.map(|s| match coding {
        Coding::Pcm16 => {
            (f32::from(i16::from_le_bytes([s[0], s[1]])) / f32::from(i16::MAX)).max(-1.0)
        }
        Coding::Float32 => clean(f32::from_le_bytes([s[0], s[1], s[2], s[3]])),
    }))
}

/// A float sample within -1 to 1; one that is no number is silence.
fn clean(v: f32) -> f32 {
    if v.is_nan() { 0.0 } else { v.clamp(-1.0, 1.0) }
}

/// The little-endian `u16` at `at`, if the bytes are there.
fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    let two = b.get(at..at.checked_add(2)?)?;
    Some(u16::from_le_bytes(two.try_into().ok()?))
}

/// The little-endian `u32` at `at`, if the bytes are there.
fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    let four = b.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes(four.try_into().ok()?))
}

#[cfg(test)]
mod tests;
