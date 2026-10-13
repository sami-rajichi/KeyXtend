//! The strict reader gives a shipped sound's sample bytes and refuses anything else as an error, never a panic.

use kx_theme::sounds::Format;
use kx_theme::sounds::wav::{DATA, HEADER_SLACK, PCM, WavError, check_size, samples};

/// Format tags of IEEE float and of the extensible header.
const FLOAT: u16 = 3;
const EXTENSIBLE: u16 = 0xFFFE;
/// Where the byte rate sits in a file `wav` writes.
const BYTE_RATE_AT: usize = 12 + 8 + 8;
/// The small format's rate.
pub const RATE: u32 = 8_000;
/// Eight 16-bit mono frames: the longest sound `format` allows.
const FULL: [u8; 16] = [1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7, 0, 8, 0];

/// A small format: 16-bit mono at 8 kHz, at most 1 ms (8 frames).
pub fn format() -> Format {
    Format {
        rate: RATE,
        bits: 16,
        channels: 1,
        ceiling_db: -3.0,
        fade_ms: 0.0,
        trim_db: -50.0,
        max_ms: 1.0,
    }
}

/// A RIFF/WAVE file holding `chunks`, each padded to an even length.
pub fn riff(chunks: &[(&[u8; 4], &[u8])]) -> Vec<u8> {
    let size = |len: usize| {
        u32::try_from(len)
            .expect("a test file is small")
            .to_le_bytes()
    };
    let mut body = b"WAVE".to_vec();
    for (id, content) in chunks {
        body.extend(*id);
        body.extend(size(content.len()));
        body.extend(*content);
        if content.len() % 2 == 1 {
            body.push(0);
        }
    }
    let mut out = b"RIFF".to_vec();
    out.extend(size(body.len()));
    out.extend(body);
    out
}

/// A basic `fmt ` chunk's content.
pub fn fmt(tag: u16, channels: u16, rate: u32, bits: u16) -> Vec<u8> {
    let align = channels * bits / 8;
    let mut out = Vec::new();
    out.extend(tag.to_le_bytes());
    out.extend(channels.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * u32::from(align)).to_le_bytes());
    out.extend(align.to_le_bytes());
    out.extend(bits.to_le_bytes());
    out
}

/// A WAV file of `tag`, `channels`, `rate` and `bits` holding `data`.
pub fn wav(tag: u16, channels: u16, rate: u32, bits: u16, data: &[u8]) -> Vec<u8> {
    riff(&[(b"fmt ", &fmt(tag, channels, rate, bits)), (b"data", data)])
}

/// A valid file of the longest sound `format` allows.
fn good() -> Vec<u8> {
    wav(PCM, 1, RATE, 16, &FULL)
}

#[test]
fn a_valid_file_gives_exactly_its_sample_bytes() {
    assert_eq!(samples(&good(), &format()), Ok(&FULL[..]));
    let fmt = fmt(PCM, 1, RATE, 16);
    let listed = riff(&[(b"LIST", &[1, 2, 3]), (b"fmt ", &fmt), (b"data", &FULL)]);
    assert_eq!(
        samples(&listed, &format()),
        Ok(&FULL[..]),
        "other chunks are skipped"
    );
}

#[test]
fn stereo_24_bit_float_and_a_wrong_rate_are_refused() {
    let cases = [
        ("stereo", wav(PCM, 2, RATE, 16, &FULL)),
        ("24-bit", wav(PCM, 1, RATE, 24, &[0; 24])),
        ("float", wav(FLOAT, 1, RATE, 32, &[0; 32])),
        ("rate", wav(PCM, 1, RATE * 2, 16, &FULL)),
    ];
    for (what, file) in cases {
        let err = samples(&file, &format()).unwrap_err();
        assert!(matches!(err, WavError::Unsupported(_)), "{what}: {err}");
    }
}

#[test]
fn a_wrong_block_size_or_a_partial_frame_is_refused() {
    let mut file = good();
    let align_at = 12 + 8 + 12;
    file[align_at..align_at + 2].copy_from_slice(&4u16.to_le_bytes());
    assert!(matches!(
        samples(&file, &format()),
        Err(WavError::Malformed(_))
    ));
    let odd = wav(PCM, 1, RATE, 16, &FULL[..15]);
    assert!(matches!(
        samples(&odd, &format()),
        Err(WavError::Malformed(_))
    ));
}

#[test]
fn every_truncation_is_refused() {
    let file = good();
    for len in 0..file.len() {
        assert!(samples(&file[..len], &format()).is_err(), "{len} bytes");
    }
}

#[test]
fn an_oversized_chunk_or_riff_size_is_refused() {
    let mut data = good();
    let at = data.len() - FULL.len() - 4;
    data[at..at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(samples(&data, &format()), Err(WavError::Truncated));
    let mut whole = good();
    let len = u32::try_from(whole.len()).unwrap();
    whole[4..8].copy_from_slice(&len.to_le_bytes());
    assert_eq!(samples(&whole, &format()), Err(WavError::Truncated));
}

#[test]
fn a_file_without_a_data_chunk_is_refused() {
    let file = riff(&[(b"fmt ", &fmt(PCM, 1, RATE, 16))]);
    assert_eq!(samples(&file, &format()), Err(WavError::Missing(DATA)));
}

#[test]
fn a_sound_with_no_sample_is_refused_as_it_would_play_nothing() {
    let err = samples(&wav(PCM, 1, RATE, 16, &[]), &format()).unwrap_err();
    assert!(matches!(err, WavError::Malformed(_)), "{err}");
}

#[test]
fn a_file_over_the_length_limit_is_refused() {
    let long = [&FULL[..], &[9, 0]].concat();
    let err = samples(&wav(PCM, 1, RATE, 16, &long), &format()).unwrap_err();
    assert!(matches!(err, WavError::TooLong(_)), "{err}");
    assert!(err.to_string().contains("1 ms"), "{err}");
}

#[test]
fn an_extended_fmt_chunk_or_a_wrong_byte_rate_is_refused() {
    let mut ext = fmt(EXTENSIBLE, 1, RATE, 16);
    // The extension's size, valid bits, channel mask, then a sub-format whose tag is PCM.
    ext.extend([22, 0, 16, 0, 4, 0, 0, 0]);
    ext.extend(PCM.to_le_bytes());
    ext.extend([0; 14]);
    let mut long = fmt(PCM, 1, RATE, 16);
    long.extend([0, 0]);
    for (what, f) in [("extensible", ext), ("18-byte fmt", long)] {
        let err = samples(&riff(&[(b"fmt ", &f), (b"data", &FULL)]), &format()).unwrap_err();
        assert!(matches!(err, WavError::Unsupported(_)), "{what}: {err}");
    }
    let mut file = good();
    file[BYTE_RATE_AT..BYTE_RATE_AT + 4].copy_from_slice(&RATE.to_le_bytes());
    let err = samples(&file, &format()).unwrap_err();
    assert!(err.to_string().contains("byte rate"), "{err}");
}

#[test]
fn data_before_fmt_or_a_repeated_chunk_is_refused() {
    let f = fmt(PCM, 1, RATE, 16);
    let cases = [
        ("data first", riff(&[(b"data", &FULL), (b"fmt ", &f)])),
        (
            "two data",
            riff(&[(b"fmt ", &f), (b"data", &FULL), (b"data", &FULL)]),
        ),
        (
            "two fmt",
            riff(&[(b"fmt ", &f), (b"fmt ", &f), (b"data", &FULL)]),
        ),
    ];
    for (what, file) in cases {
        let err = samples(&file, &format()).unwrap_err();
        assert!(matches!(err, WavError::Malformed(_)), "{what}: {err}");
    }
}

#[test]
fn bytes_past_the_riff_size_are_refused() {
    let padded = [good(), vec![0, 0]].concat();
    let chunk = [good(), b"LIST".to_vec(), vec![0; 4]].concat();
    for (what, file) in [("padding", padded), ("a chunk", chunk)] {
        let err = samples(&file, &format()).unwrap_err();
        assert!(matches!(err, WavError::Malformed(_)), "{what}: {err}");
    }
}

#[test]
fn a_file_larger_than_any_sound_is_refused_before_its_chunks_are_read() {
    let max = u64::try_from(FULL.len() + HEADER_SLACK).expect("a small size");
    assert_eq!(check_size(max, &format()), Ok(()));
    assert_eq!(check_size(max + 1, &format()), Err(WavError::TooBig(max)));
    assert!(check_size(u64::MAX, &format()).is_err());
    let notes = vec![0; HEADER_SLACK];
    let f = fmt(PCM, 1, RATE, 16);
    let big = riff(&[(b"LIST", &notes), (b"fmt ", &f), (b"data", &FULL)]);
    assert_eq!(samples(&big, &format()), Err(WavError::TooBig(max)));
}

#[test]
fn no_damaged_or_random_input_makes_it_panic() {
    let file = good();
    for at in 0..file.len() {
        for byte in [0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF] {
            let mut bad = file.clone();
            bad[at] = byte;
            let _ = samples(&bad, &format());
        }
    }
    // A fixed xorshift, so every run reads the same buffers.
    let mut x: u32 = 0x9E37_79B9;
    for len in 0..512 {
        let noise: Vec<u8> = (0..len % 96)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                x.to_le_bytes()[0]
            })
            .collect();
        let _ = samples(&[&file[..12], &noise[..]].concat(), &format());
        let _ = samples(&noise, &format());
    }
}
