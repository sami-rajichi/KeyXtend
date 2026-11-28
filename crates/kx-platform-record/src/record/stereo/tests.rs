//! Every channel reaches the stereo track by the table's downmix, scaled so it never clips, and any rate becomes the
//! track's.

use super::*;
use crate::table::table;

/// How far a mixed sample may stray from the sum worked by hand.
const CLOSE: f32 = 1e-4;
/// Input rates the resampler turns into the track's.
const RATES: [u32; 3] = [16_000, 44_100, 96_000];

/// The stereo frames `channels` at the track's rate make of `frames`, each frame one input frame.
fn mixed(channels: u16, frames: &[&[f32]]) -> Vec<[f32; 2]> {
    let mut stereo = Stereo::new(channels, RATE, table().unwrap());
    let input: Vec<f32> = frames.iter().flat_map(|f| f.iter().copied()).collect();
    // One frame more pushes the last one out, as the resampler waits for the next to blend with.
    let tail = vec![0.0; usize::from(channels)];
    let out = [stereo.push(&input), stereo.push(&tail)].concat();
    out.as_chunks::<2>().0.to_vec()
}

/// True when each pair is within `CLOSE` of the want.
fn close(got: &[[f32; 2]], want: &[[f32; 2]]) -> bool {
    got.len() == want.len()
        && got
            .iter()
            .zip(want)
            .all(|(g, w)| (g[0] - w[0]).abs() < CLOSE && (g[1] - w[1]).abs() < CLOSE)
}

#[test]
fn mono_reaches_both_sides_and_stereo_stays_as_it_is() {
    assert!(close(&mixed(1, &[&[0.5]]), &[[0.5, 0.5]]));
    assert!(close(&mixed(2, &[&[0.5, -0.25]]), &[[0.5, -0.25]]));
}

#[test]
fn five_one_keeps_every_channel_but_the_low_one_without_clipping() {
    // Each side's gains add up to 1 + 0.707 + 0.707.
    let side = 1.0 + 0.707 + 0.707;
    let full = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let got = mixed(
        6,
        &[
            &[1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            &[0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
            &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            &[0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
            &full,
        ],
    );
    let want = [
        [1.0 / side, 0.0],
        [0.707 / side, 0.707 / side],
        [0.0, 0.0],
        [0.0, 0.707 / side],
        [1.0, 1.0],
    ];
    assert!(close(&got, &want), "{got:?}");
}

#[test]
fn a_count_the_table_does_not_list_sends_the_rest_to_both_sides() {
    let got = mixed(3, &[&[1.0, 0.0, 0.0], &[0.0, 0.0, 1.0]]);
    assert!(close(&got, &[[0.5, 0.0], [0.5, 0.5]]), "{got:?}");
}

#[test]
fn the_track_rate_passes_as_it_is_one_frame_behind() {
    let mut stereo = Stereo::new(2, RATE, table().unwrap());
    let frames = [[0.0, 0.25], [0.5, -0.5], [1.0, -1.0]];
    let out: Vec<f32> = frames.iter().flat_map(|f| stereo.push(f)).collect();
    assert_eq!(out, [0.0, 0.25, 0.5, -0.5]);
}

#[test]
fn any_rate_becomes_the_track_rate() {
    for from in RATES {
        let mut stereo = Stereo::new(1, from, table().unwrap());
        let input = vec![0.0; usize::try_from(from).unwrap()];
        let made = stereo.push(&input).len() / usize::from(CHANNELS);
        let want = u64::from(from - 1) * u64::from(RATE) / u64::from(from);
        let made = u64::try_from(made).unwrap();
        assert!(
            made.abs_diff(want) <= 1,
            "{from} Hz: {made} frames, not {want}"
        );
    }
}
