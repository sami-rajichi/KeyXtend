//! WAV reading: 16-bit PCM and 32-bit float in any rate and channels, the shipped check clips, chunks skipped, a cut
//! file read as far as it holds, and every other file refused as `BadFile`.

use super::*;
use std::fs;

/// A WAV file of format `tag` with `bits` per sample, holding `data`, an odd `LIST` chunk before its samples.
fn wav(tag: u16, bits: u16, channels: u16, rate: u32, data: &[u8]) -> Vec<u8> {
    let mut fmt = Vec::new();
    for v in [tag, channels] {
        fmt.extend_from_slice(&v.to_le_bytes());
    }
    let block = channels * bits / BYTE_BITS;
    fmt.extend_from_slice(&rate.to_le_bytes());
    fmt.extend_from_slice(&(rate * u32::from(block)).to_le_bytes());
    fmt.extend_from_slice(&block.to_le_bytes());
    fmt.extend_from_slice(&bits.to_le_bytes());
    let mut body = WAVE_ID.to_vec();
    for (id, part) in [(FMT_ID, &fmt[..]), (b"LIST", b"abc"), (DATA_ID, data)] {
        body.extend_from_slice(id);
        body.extend_from_slice(&u32::try_from(part.len()).unwrap().to_le_bytes());
        body.extend_from_slice(part);
        if part.len() % 2 == 1 {
            body.push(0);
        }
    }
    let mut out = RIFF_ID.to_vec();
    out.extend_from_slice(&u32::try_from(body.len()).unwrap().to_le_bytes());
    out.extend_from_slice(&body);
    out
}

fn pcm(samples: &[i16]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

fn float(samples: &[f32]) -> Vec<u8> {
    samples.iter().flat_map(|s| s.to_le_bytes()).collect()
}

#[test]
fn sixteen_bit_pcm_reads_as_floats_with_its_format() {
    let bytes = wav(PCM, 16, 2, 48_000, &pcm(&[0, i16::MAX, i16::MIN, 16_384]));
    let sound = parse(&bytes).unwrap();
    let want = Format {
        rate: 48_000,
        channels: 2,
    };
    assert_eq!(sound.format, want);
    assert_eq!(&sound.samples[..3], [0.0, 1.0, -1.0]);
    assert!((sound.samples[3] - 0.5).abs() < 1e-4);
}

#[test]
fn float_samples_are_kept_and_cleaned() {
    let data = float(&[0.25, -0.5, 2.0, f32::NAN, f32::NEG_INFINITY]);
    let sound = parse(&wav(FLOAT, 32, 1, 16_000, &data)).unwrap();
    assert_eq!(&sound.samples[..], [0.25, -0.5, 1.0, 0.0, -1.0]);
}

#[test]
fn a_cut_file_reads_its_whole_frames_only() {
    let mut bytes = wav(PCM, 16, 2, 16_000, &pcm(&[1, 2, 3, 4, 5]));
    bytes.truncate(bytes.len() - 1);
    assert_eq!(
        parse(&bytes).unwrap().samples.len(),
        4,
        "a frame of two samples each"
    );
}

#[test]
fn other_codings_and_broken_files_are_refused() {
    let one = pcm(&[0]);
    let cases = [
        wav(PCM, 8, 1, 16_000, &one),
        wav(PCM, 24, 1, 16_000, &one),
        wav(FLOAT, 64, 1, 16_000, &one),
        wav(0xFFFE, 16, 1, 16_000, &one),
        wav(PCM, 16, 0, 16_000, &one),
        wav(PCM, 16, 1, 0, &one),
        b"not a wav file at all, only some words".to_vec(),
        Vec::new(),
    ];
    for (k, bytes) in cases.iter().enumerate() {
        assert_eq!(parse(bytes).err(), Some(VoiceFault::BadFile), "case {k}");
    }
    let mut no_data = wav(PCM, 16, 1, 16_000, &one);
    let at = no_data.windows(4).position(|w| w == DATA_ID).unwrap();
    no_data.truncate(at);
    assert_eq!(parse(&no_data).err(), Some(VoiceFault::BadFile));
}

/// `bytes` with the little-endian `v` written at `at`.
fn put(mut bytes: Vec<u8>, at: usize, v: &[u8]) -> Vec<u8> {
    bytes[at..at + v.len()].copy_from_slice(v);
    bytes
}

#[test]
fn hostile_headers_are_refused_without_a_panic() {
    let good = wav(PCM, 16, 1, 16_000, &pcm(&[1, 2]));
    let fmt_size = FIRST_CHUNK + ID_LEN;
    let body = FIRST_CHUNK + CHUNK_HEAD;
    let cases = [
        put(good.clone(), fmt_size, &u32::MAX.to_le_bytes()),
        put(good.clone(), fmt_size, &4_u32.to_le_bytes()),
        put(good.clone(), fmt_size, &0_u32.to_le_bytes()),
        put(good.clone(), body + RATE_AT, &u32::MAX.to_le_bytes()),
        put(
            good.clone(),
            body + RATE_AT,
            &(RATES.start() - 1).to_le_bytes(),
        ),
        put(good.clone(), body + CHANNELS_AT, &u16::MAX.to_le_bytes()),
        put(
            good.clone(),
            body + CHANNELS_AT,
            &(MAX_CHANNELS + 1).to_le_bytes(),
        ),
        put(good.clone(), WAVE_AT, b"AVI "),
        good[..FIRST_CHUNK - 1].to_vec(),
        good[..body + FMT_LEAST - 1].to_vec(),
    ];
    for (k, bytes) in cases.iter().enumerate() {
        assert_eq!(parse(bytes).err(), Some(VoiceFault::BadFile), "case {k}");
    }
}

#[test]
fn a_data_size_past_the_file_reads_what_it_holds_and_a_later_data_chunk_is_ignored() {
    let data = pcm(&[7, 8]);
    let mut bytes = wav(PCM, 16, 1, 16_000, &data);
    let size_at = bytes.len() - data.len() - ID_LEN;
    bytes = put(bytes, size_at, &u32::MAX.to_le_bytes());
    assert_eq!(parse(&bytes).unwrap().samples.len(), 2);
    let mut twice = wav(PCM, 16, 1, 16_000, &pcm(&[i16::MAX]));
    twice.extend_from_slice(DATA_ID);
    twice.extend_from_slice(&2_u32.to_le_bytes());
    twice.extend_from_slice(&pcm(&[0]));
    assert_eq!(&parse(&twice).unwrap().samples[..], [1.0]);
}

#[test]
fn the_rates_and_channels_at_the_limits_are_taken() {
    for (rate, channels) in [(*RATES.start(), 1), (*RATES.end(), MAX_CHANNELS)] {
        let one_frame = pcm(&vec![0; usize::from(channels)]);
        let sound = parse(&wav(PCM, 16, channels, rate, &one_frame)).unwrap();
        assert_eq!(
            (sound.format.rate, sound.samples.len()),
            (rate, usize::from(channels))
        );
    }
}

proptest::proptest! {
    #[test]
    fn random_or_changed_bytes_never_panic(
        noise in proptest::collection::vec(proptest::num::u8::ANY, 0..256),
        at in 0_usize..64,
    ) {
        drop(parse(&noise));
        let mut changed = wav(FLOAT, 32, 2, 48_000, &float(&[0.5; 8]));
        for (k, b) in noise.iter().enumerate() {
            if let Some(slot) = changed.get_mut(at + k) {
                *slot = *b;
            }
        }
        drop(parse(&changed));
    }
}

#[test]
fn the_shipped_check_clips_read_as_six_seconds_of_16_khz_mono_float() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/voice");
    for lang in ["en", "fr", "ar"] {
        let sound = read(&dir.join(format!("check-{lang}.wav"))).unwrap();
        let want = Format {
            rate: 16_000,
            channels: 1,
        };
        assert_eq!(sound.format, want, "{lang}");
        let secs = sound.samples.len() / 16_000;
        assert!((5..=6).contains(&secs), "{lang}: {secs} s");
        assert!(
            sound.samples.iter().any(|&s| s != 0.0),
            "{lang} holds sound"
        );
    }
}

#[test]
fn a_missing_file_is_no_file_and_one_past_the_cap_is_refused() {
    let dir = std::env::temp_dir().join(format!("kx-wav-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    assert_eq!(read(&dir.join("none.wav")).err(), Some(VoiceFault::NoFile));
    let big = dir.join("big.wav");
    let file = File::create(&big).unwrap();
    file.set_len(MAX_WAV_BYTES + 1).unwrap();
    drop(file);
    assert_eq!(read(&big).err(), Some(VoiceFault::BadFile));
    fs::remove_dir_all(&dir).unwrap();
}
