//! The spike's fixtures, the usual microphone formats to 16 kHz mono, and a property: any chunking gives exactly the
//! samples of the plain formula over the whole buffer.

use super::*;
use kx_ipc::SOUND_HZ;
use proptest::prelude::*;
use std::cmp::Ordering;
use std::f32::consts::TAU;

/// The plain formula over a whole buffer, as the spike's `to_rate` computed it with whole-number positions.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn reference(x: &[f32], from: u64, to: u64) -> Vec<f32> {
    let len = x.len() as u64;
    let at = |n: u64| x[n as usize];
    (0..len * to / from)
        .map(|i| {
            let a = i * from / to;
            match from.cmp(&to) {
                Ordering::Greater => {
                    let b = ((i + 1) * from / to).min(len).max(a + 1);
                    let w = &x[a as usize..b as usize];
                    w.iter().sum::<f32>() / w.len() as f32
                }
                Ordering::Equal => at(a),
                Ordering::Less => {
                    let t = ((i * from) % to) as f32 / to as f32;
                    at(a) + (at((a + 1).min(len - 1)) - at(a)) * t
                }
            }
        })
        .collect()
}

/// `x` from `from` to `to` Hz, fed in chunks of the sizes in `cuts`, in turn.
fn stream(x: &[f32], from: u32, to: usize, cuts: &[usize]) -> Vec<f32> {
    let mut r = Resampler::new(from, to);
    let (mut out, mut rest, mut k) = (Vec::new(), x, 0);
    while !rest.is_empty() {
        let (chunk, tail) = rest.split_at(cuts[k % cuts.len()].min(rest.len()));
        out.extend_from_slice(&r.push(chunk).unwrap());
        (rest, k) = (tail, k + 1);
    }
    out.extend_from_slice(&r.flush().unwrap());
    out
}

/// `x` from `from` Hz to 16 kHz in one chunk.
fn whole(x: &[f32], from: u32) -> Vec<f32> {
    stream(x, from, SOUND_HZ, &[usize::MAX])
}

/// Times the signal crosses zero going up.
fn rises(x: &[f32]) -> usize {
    x.windows(2).filter(|w| w[0] < 0.0 && w[1] >= 0.0).count()
}

/// `frames` of a `hz` sine at `rate`, interleaved in `channels` equal channels.
#[allow(clippy::cast_precision_loss)]
fn sine(hz: f32, rate: u32, frames: usize, channels: usize) -> Vec<f32> {
    (0..frames)
        .flat_map(|i| vec![(TAU * hz * i as f32 / rate as f32).sin() * 0.5; channels])
        .collect()
}

#[test]
fn stereo_is_averaged_to_mono_and_a_cut_frame_waits() {
    assert_eq!(
        &*Mixer::new(2).take(&[1.0, 3.0, 2.0, 4.0]).unwrap(),
        &[2.0, 3.0]
    );
    assert_eq!(&*Mixer::new(1).take(&[0.5, 0.25]).unwrap(), &[0.5, 0.25]);
    assert_eq!(&*Mixer::new(0).take(&[0.5]).unwrap(), &[0.5]);
    let mut m = Mixer::new(2);
    assert!(m.take(&[1.0]).unwrap().is_empty());
    assert_eq!(&*m.take(&[3.0, 2.0]).unwrap(), &[2.0]);
    assert_eq!(&*m.take(&[4.0]).unwrap(), &[3.0]);
}

#[test]
fn common_mic_rates_become_16k() {
    for (from, n) in [(48_000, 4800), (44_100, 4410), (16_000, 1600), (8000, 800)] {
        let out = whole(&vec![0.5; n], from);
        assert_eq!(out.len(), 1600, "from {from} Hz");
        assert!(out.iter().all(|&y| (y - 0.5).abs() < 1e-6), "from {from}");
    }
    assert!(whole(&[], 48_000).is_empty());
}

#[test]
fn a_ramp_keeps_its_shape() {
    let ramp: Vec<f32> = (0_u8..12).map(f32::from).collect();
    assert_eq!(stream(&ramp, 48_000, 16_000, &[5]), [1.0, 4.0, 7.0, 10.0]);
    let up = stream(&[0.0, 2.0, 4.0], 8000, 16_000, &[1]);
    assert_eq!(up, [0.0, 1.0, 2.0, 3.0, 4.0, 4.0]);
}

#[test]
fn a_48k_stereo_tone_becomes_the_same_tone_at_16k_mono() {
    let frames = sine(1000.0, 48_000, 4800, 2);
    let mono = Mixer::new(2).take(&frames).unwrap();
    let out = whole(&mono, 48_000);
    assert_eq!(out.len(), 1600);
    assert!((99..=101).contains(&rises(&out)), "{}", rises(&out));
}

#[test]
fn a_44k1_tone_becomes_the_same_tone_at_16k() {
    let out = whole(&sine(440.0, 44_100, 44_100, 1), 44_100);
    assert_eq!(out.len(), SOUND_HZ);
    assert!((439..=441).contains(&rises(&out)), "{}", rises(&out));
}

#[test]
fn pcm16_rounds_and_clamps() {
    let out = pcm16(&[0.0, 0.5, 2.0, -2.0, f32::NAN]).unwrap();
    assert_eq!(&*out, &[0, 16384, 32767, -32767, 0]);
}

proptest! {
    #[test]
    fn any_chunking_gives_the_whole_buffers_samples(
        x in prop::collection::vec(-1.0_f32..1.0, 0..2500),
        from in prop_oneof![Just(8000_u32), Just(16_000), Just(44_100), Just(48_000), 4000_u32..200_000],
        to in prop_oneof![Just(SOUND_HZ), 4000_usize..100_000],
        cuts in prop::collection::vec(1_usize..400, 1..8),
    ) {
        let want = reference(&x, u64::from(from), to as u64);
        prop_assert_eq!(stream(&x, from, to, &cuts), want);
    }

    #[test]
    fn a_mixer_gives_the_same_mono_however_its_chunks_fall(
        x in prop::collection::vec(-1.0_f32..1.0, 0..600),
        channels in 1_u16..8,
        cuts in prop::collection::vec(1_usize..50, 1..6),
    ) {
        let n = usize::from(channels);
        let whole: Vec<f32> = x.chunks_exact(n).map(|f| f.iter().sum::<f32>() / f32::from(channels)).collect();
        let (mut m, mut out, mut rest, mut k) = (Mixer::new(channels), Vec::new(), &x[..], 0);
        while !rest.is_empty() {
            let (chunk, tail) = rest.split_at(cuts[k % cuts.len()].min(rest.len()));
            out.extend_from_slice(&m.take(chunk).unwrap());
            (rest, k) = (tail, k + 1);
        }
        prop_assert_eq!(out, whole);
    }
}
