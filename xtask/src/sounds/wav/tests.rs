use proptest::prelude::*;

use super::*;
use crate::sounds::fixture::{TAG_FLOAT, TAG_PCM, encode, header, mono};

/// A short valid 16-bit mono file.
fn small() -> Vec<u8> {
    encode(TAG_PCM, 8_000, 16, &mono(&[0.0, 0.5, -0.5, 0.25]))
}

#[test]
fn written_file_has_a_pcm16_mono_header_and_reads_back() {
    let wav = write(48_000, &[0, 16_384, -16_384, i16::MAX]).unwrap();
    assert_eq!(header(&wav), (1, 1, 48_000, 16, 8));
    let audio = read(&wav).unwrap();
    assert_eq!(audio.rate, 48_000);
    assert_eq!(
        audio.samples,
        [0.0, 0.5, -0.5, f64::from(i16::MAX) / 32_768.0]
    );
}

#[test]
fn reads_8_16_24_bit_integers_and_32_bit_float() {
    let values = [0.0, 0.5, -0.5, -1.0];
    for (tag, bits) in [(TAG_PCM, 8), (TAG_PCM, 16), (TAG_PCM, 24), (TAG_FLOAT, 32)] {
        let audio = read(&encode(tag, 44_100, bits, &mono(&values))).unwrap();
        assert_eq!(audio.rate, 44_100);
        assert_eq!(audio.samples, values, "{bits} bits");
    }
}

#[test]
fn stereo_mixes_to_mono() {
    let frames = vec![vec![0.5, 0.25], vec![-1.0, 0.0]];
    let audio = read(&encode(TAG_PCM, 48_000, 24, &frames)).unwrap();
    assert_eq!(audio.samples, [0.375, -0.5]);
}

#[test]
fn an_extensible_header_naming_pcm_is_read() {
    let mut wav = small();
    let extra = [
        22u8, 0, 16, 0, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113,
    ];
    wav[20..22].copy_from_slice(&0xFFFEu16.to_le_bytes());
    wav[16..20].copy_from_slice(&40u32.to_le_bytes());
    let tail = wav.split_off(36);
    wav.extend(extra);
    wav.extend(tail);
    let riff = u32::try_from(wav.len() - 8).unwrap();
    wav[4..8].copy_from_slice(&riff.to_le_bytes());
    assert_eq!(read(&wav).unwrap().samples, [0.0, 0.5, -0.5, 0.25]);
}

#[test]
fn other_chunks_are_skipped_with_their_pad_byte() {
    let mut wav = small();
    let tail = wav.split_off(12);
    wav.extend(b"LIST");
    wav.extend(3u32.to_le_bytes());
    wav.extend([1, 2, 3, 0]);
    wav.extend(tail);
    let riff = u32::try_from(wav.len() - 8).unwrap();
    wav[4..8].copy_from_slice(&riff.to_le_bytes());
    assert_eq!(read(&wav).unwrap().samples.len(), 4);
}

#[test]
fn a_short_file_is_named_by_its_kind() {
    assert_eq!(read(b"not a wav").unwrap_err(), WavError::NotWave);
    assert_eq!(read(&small()[..10]).unwrap_err(), WavError::Truncated);
}

#[test]
fn every_truncation_is_refused() {
    let wav = small();
    for len in 0..wav.len() {
        assert!(read(&wav[..len]).is_err(), "a {len}-byte prefix was read");
    }
}

#[test]
fn malformed_headers_are_refused() {
    let edits: [(usize, &[u8]); 7] = [
        (0, b"RIFX"),
        (8, b"WAVX"),
        (12, b"fmtX"),
        (22, &3u16.to_le_bytes()),
        (34, &12u16.to_le_bytes()),
        (32, &4u16.to_le_bytes()),
        (24, &0u32.to_le_bytes()),
    ];
    for (at, bytes) in edits {
        let mut wav = small();
        wav[at..at + bytes.len()].copy_from_slice(bytes);
        assert!(read(&wav).is_err(), "edit at {at} was read");
    }
}

#[test]
fn a_partial_frame_or_a_nan_sample_is_refused() {
    let mut wav = small();
    wav.push(0);
    wav[40..44].copy_from_slice(&9u32.to_le_bytes());
    let riff = u32::try_from(wav.len() - 8).unwrap();
    wav[4..8].copy_from_slice(&riff.to_le_bytes());
    assert!(read(&wav).is_err());
    let nan = encode(TAG_FLOAT, 48_000, 32, &mono(&[0.0, f64::NAN]));
    assert!(read(&nan).is_err());
}

proptest! {
    #[test]
    fn read_never_panics_on_any_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..128)) {
        let _ = read(&bytes);
    }

    #[test]
    fn read_never_panics_on_a_damaged_file(at in 0usize..52, byte in any::<u8>(), cut in 0usize..60) {
        let mut wav = small();
        let at = at % wav.len();
        wav[at] = byte;
        wav.truncate(cut.max(1));
        let _ = read(&wav);
    }
}

#[test]
fn a_file_without_a_data_chunk_is_refused() {
    let mut wav = small();
    wav.truncate(36);
    let riff = u32::try_from(wav.len() - 8).unwrap();
    wav[4..8].copy_from_slice(&riff.to_le_bytes());
    assert_eq!(read(&wav).unwrap_err(), WavError::Missing(DATA));
}

#[test]
fn zero_channels_are_refused() {
    let mut wav = small();
    wav[22..24].copy_from_slice(&0u16.to_le_bytes());
    assert!(matches!(read(&wav).unwrap_err(), WavError::Unsupported(_)));
}

#[test]
fn a_read_reports_the_file_layout() {
    let audio = read(&small()).unwrap();
    assert_eq!((audio.rate, audio.channels, audio.bits), (8_000, 1, 16));
    let stereo = encode(TAG_PCM, 44_100, 24, &[vec![0.5, 0.25]]);
    let audio = read(&stereo).unwrap();
    assert_eq!((audio.rate, audio.channels, audio.bits), (44_100, 2, 24));
}
