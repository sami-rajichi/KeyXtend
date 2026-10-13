//! Test fixtures: a small recipe book and helpers that build and inspect WAV files.

use std::f64::consts::TAU;

/// A small valid recipe book: one recipe with three variants, one without.
pub(crate) const BOOK: &str = r#"
[format]
rate = 48000
bits = 16
channels = 1
ceiling_db = -3.0
fade_ms = 5
trim_db = -50.0
max_ms = 400

[variants]
pitch_cents = 40
level_db = 1.5

[resample]
half_taps = 32
rolloff = 0.94

[[recipe]]
name = "tap"
length_ms = 60
gain_db = 0
variants = 3

[[recipe.layer]]
kind = "noise"
filter = "bandpass"
hz = 3000
q = 1.0
start_ms = 0
attack_ms = 0.3
decay_ms = 5
level = 0.5

[[recipe.layer]]
kind = "modes"
start_ms = 1
attack_ms = 0.5
level = 1.0
glide = { ratio = 1.2, ms = 20 }
partials = [
    { hz = 800, decay_ms = 30, level = 1.0 },
    { hz = 1300, decay_ms = 20, level = 0.5 },
]

[[recipe]]
name = "thud"
length_ms = 100
gain_db = -2

[[recipe.layer]]
kind = "noise"
filter = "lowpass"
hz = 500
q = 0.7
start_ms = 2
attack_ms = 1
decay_ms = 40
level = 1.0
glide = { ratio = 0.8, ms = 30 }
"#;

/// WAV format tag of integer PCM.
pub(crate) const TAG_PCM: u16 = 1;
/// WAV format tag of IEEE float.
pub(crate) const TAG_FLOAT: u16 = 3;
/// Bytes of the header our writer makes, before the samples.
pub(crate) const HEADER: usize = 44;

/// The header fields our writer sets: tag, channels, rate, bits and data length.
pub(crate) fn header(wav: &[u8]) -> (u16, u16, u32, u16, u32) {
    let u16_at = |at: usize| u16::from_le_bytes([wav[at], wav[at + 1]]);
    let u32_at = |at: usize| u32::from_le_bytes([wav[at], wav[at + 1], wav[at + 2], wav[at + 3]]);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(u32_at(4) as usize, wav.len() - 8);
    assert_eq!(&wav[8..16], b"WAVEfmt ");
    assert_eq!(&wav[36..40], b"data");
    (u16_at(20), u16_at(22), u32_at(24), u16_at(34), u32_at(40))
}

/// The 16-bit samples of a file our writer made.
pub(crate) fn pcm(wav: &[u8]) -> Vec<i16> {
    let (pairs, _) = wav[HEADER..].as_chunks::<2>();
    pairs.iter().map(|b| i16::from_le_bytes(*b)).collect()
}

/// The largest magnitude among `samples`.
pub(crate) fn peak(samples: &[i16]) -> i32 {
    samples
        .iter()
        .map(|s| i32::from(*s).abs())
        .max()
        .unwrap_or(0)
}

/// Sign changes in `samples`, skipping zeros.
pub(crate) fn crossings(samples: &[i16]) -> usize {
    let signs: Vec<bool> = samples
        .iter()
        .filter(|s| **s != 0)
        .map(|s| *s > 0)
        .collect();
    signs.windows(2).filter(|w| w[0] != w[1]).count()
}

/// A sine of `hz` at `rate`, `ms` long, with `amp` amplitude, starting at phase zero.
pub(crate) fn sine(hz: f64, rate: u32, ms: u32, amp: f64) -> Vec<f64> {
    let n = rate / 1000 * ms;
    (0..n)
        .map(|i| amp * (TAU * hz * f64::from(i) / f64::from(rate)).sin())
        .collect()
}

/// A WAV file in any layout: `frames` holds one value per channel, from -1 to 1.
pub(crate) fn encode(tag: u16, rate: u32, bits: u16, frames: &[Vec<f64>]) -> Vec<u8> {
    let channels = u16::try_from(frames[0].len()).unwrap();
    let align = channels * bits / 8;
    let mut data = Vec::new();
    for frame in frames {
        for value in frame {
            data.extend(sample_bytes(tag, bits, *value));
        }
    }
    let mut out = Vec::new();
    out.extend(b"RIFF");
    out.extend(
        u32::try_from(4 + 8 + 16 + 8 + data.len())
            .unwrap()
            .to_le_bytes(),
    );
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(tag.to_le_bytes());
    out.extend(channels.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * u32::from(align)).to_le_bytes());
    out.extend(align.to_le_bytes());
    out.extend(bits.to_le_bytes());
    out.extend(b"data");
    out.extend(u32::try_from(data.len()).unwrap().to_le_bytes());
    out.extend(data);
    out
}

/// One sample's bytes in the given tag and width.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Test values are within range."
)]
fn sample_bytes(tag: u16, bits: u16, value: f64) -> Vec<u8> {
    if tag == TAG_FLOAT {
        return (value as f32).to_le_bytes().to_vec();
    }
    let full = f64::from(1u32 << (bits - 1));
    let int = (value * full).round().clamp(-full, full - 1.0) as i32;
    let bytes = usize::from(bits / 8);
    if bits == 8 {
        return vec![(int + 128) as u8];
    }
    int.to_le_bytes()[..bytes].to_vec()
}

/// One channel's samples as mono frames.
pub(crate) fn mono(samples: &[f64]) -> Vec<Vec<f64>> {
    samples.iter().map(|s| vec![*s]).collect()
}
