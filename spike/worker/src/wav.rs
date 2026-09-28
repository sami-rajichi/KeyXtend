//! 16-bit PCM WAV in memory: the clip format both engines take.

/// Bytes before the samples in the files we write.
pub const HEADER_LEN: usize = 44;
/// Where the first chunk starts, after `RIFF`, the size and `WAVE`.
const FIRST_CHUNK: usize = 12;
/// Where `WAVE` sits in the RIFF header.
const WAVE_AT: usize = 8;
/// Where our files hold the size of their `data` chunk.
#[cfg(test)]
const DATA_SIZE_AT: usize = 40;
/// Offsets in the `fmt ` chunk body of the sample rate and the block size.
const FMT_RATE_AT: usize = 4;
/// See `FMT_RATE_AT`.
const FMT_BLOCK_AT: usize = 12;
/// Bytes of a chunk id.
const ID_LEN: usize = 4;
/// Bytes of a chunk's id and size.
const CHUNK_HEAD: usize = 8;
/// Bytes of the `fmt ` chunk body for plain PCM.
const FMT_LEN: u32 = 16;
/// Format tag of plain PCM.
const PCM: u16 = 1;
/// Mono.
const MONO: u16 = 1;
/// Bytes per 16-bit sample.
const SAMPLE_BYTES: u16 = 2;
/// Bits per sample.
const BITS: u16 = 16;
/// Milliseconds in a second.
const MS_PER_S: u64 = 1000;
/// Id of the file's outer chunk.
const RIFF_ID: &[u8] = b"RIFF";
/// Form type of a WAV file, right after the RIFF size.
const WAVE_ID: &[u8] = b"WAVE";
/// Id of the chunk that holds the rate and block size.
const FMT_ID: &[u8] = b"fmt ";
/// Id of the chunk that holds the samples.
const DATA_ID: &[u8] = b"data";

/// A mono 16-bit WAV file of `samples` at `rate` Hz.
pub fn encode(samples: &[i16], rate: u32) -> Vec<u8> {
    let data = u32::try_from(samples.len() * usize::from(SAMPLE_BYTES)).unwrap_or(u32::MAX);
    let mut w = Vec::with_capacity(HEADER_LEN + data as usize);
    w.extend_from_slice(RIFF_ID);
    w.extend_from_slice(
        &(data.saturating_add(HEADER_LEN as u32 - CHUNK_HEAD as u32)).to_le_bytes(),
    );
    w.extend_from_slice(WAVE_ID);
    w.extend_from_slice(FMT_ID);
    w.extend_from_slice(&FMT_LEN.to_le_bytes());
    w.extend_from_slice(&PCM.to_le_bytes());
    w.extend_from_slice(&MONO.to_le_bytes());
    w.extend_from_slice(&rate.to_le_bytes());
    w.extend_from_slice(&(rate * u32::from(SAMPLE_BYTES)).to_le_bytes());
    w.extend_from_slice(&SAMPLE_BYTES.to_le_bytes());
    w.extend_from_slice(&BITS.to_le_bytes());
    w.extend_from_slice(DATA_ID);
    w.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        w.extend_from_slice(&s.to_le_bytes());
    }
    w
}

/// Size of the WAV file `encode` makes for `secs` seconds at `rate` Hz.
pub fn max_len(secs: u64, rate: u32) -> u64 {
    let data = secs
        .saturating_mul(u64::from(rate))
        .saturating_mul(u64::from(SAMPLE_BYTES));
    data.saturating_add(HEADER_LEN as u64)
}

/// Length of the sound in a WAV file, in ms.
pub fn audio_ms(bytes: &[u8]) -> Result<u64, String> {
    let wave = WAVE_AT..WAVE_AT + ID_LEN;
    if bytes.get(..ID_LEN) != Some(RIFF_ID) || bytes.get(wave) != Some(WAVE_ID) {
        return Err("not a WAV file".to_string());
    }
    let (mut at, mut fmt, mut data) = (FIRST_CHUNK, None, None);
    let head = |at: usize| {
        let id = bytes.get(at..at.saturating_add(ID_LEN));
        id.zip(u32_at(bytes, at.saturating_add(ID_LEN)))
    };
    while let Some((id, size)) = head(at) {
        let body = at + CHUNK_HEAD;
        match id {
            FMT_ID => {
                fmt = u32_at(bytes, body + FMT_RATE_AT).zip(u16_at(bytes, body + FMT_BLOCK_AT));
            }
            // A streamed file may declare more than it holds.
            DATA_ID => data = Some(u64::from(size).min((bytes.len() - body) as u64)),
            _ => {}
        }
        at = body.saturating_add(size as usize + (size as usize & 1));
    }
    match (fmt, data) {
        (Some((rate, block)), Some(len)) if rate > 0 && block > 0 => {
            Ok(len / u64::from(block) * MS_PER_S / u64::from(rate))
        }
        (Some(_), Some(_)) => Err("the WAV rate or block size is zero".to_string()),
        _ => Err("the WAV file has no format or no data".to_string()),
    }
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
mod tests {
    use super::*;

    #[test]
    fn a_second_at_16k_is_a_44_byte_header_and_32000_bytes() {
        let w = encode(&vec![0i16; 16000], 16000);
        assert_eq!(w.len(), HEADER_LEN + 32000);
        assert_eq!(&w[..ID_LEN], RIFF_ID);
        assert_eq!(&w[WAVE_AT..FIRST_CHUNK], WAVE_ID);
        assert_eq!(&w[FIRST_CHUNK..FIRST_CHUNK + ID_LEN], FMT_ID);
        assert_eq!(audio_ms(&w), Ok(1000));
        assert_eq!(max_len(1, 16000), w.len() as u64);
    }

    #[test]
    fn samples_are_little_endian() {
        let w = encode(&[0x1234, -2], 8000);
        assert_eq!(&w[HEADER_LEN..], &[0x34, 0x12, 0xFE, 0xFF]);
    }

    #[test]
    fn other_chunks_are_skipped_and_a_short_file_counts_what_it_holds() {
        let w = encode(&[0; 16000], 16000);
        let mut with_list = w[..FIRST_CHUNK].to_vec();
        with_list.extend_from_slice(b"LIST\x03\0\0\0abc\0");
        with_list.extend_from_slice(&w[FIRST_CHUNK..]);
        assert_eq!(
            audio_ms(&with_list),
            Ok(1000),
            "odd chunk with its pad byte"
        );
        let mut streamed = w.clone();
        streamed[DATA_SIZE_AT..HEADER_LEN].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            audio_ms(&streamed),
            Ok(1000),
            "a declared size past the end is cut to the bytes held"
        );
    }

    #[test]
    fn a_file_that_is_not_wav_is_an_error() {
        assert!(audio_ms(b"not a wav file at all, just some text here....").is_err());
        assert!(audio_ms(&[]).is_err());
        let mut w = encode(&[0; 10], 16000);
        w[24..28].copy_from_slice(&0u32.to_le_bytes());
        assert!(audio_ms(&w).is_err(), "a zero rate is refused");
    }
}
