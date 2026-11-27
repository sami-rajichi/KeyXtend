//! The run's sound: each source lands where it was heard, so a sound and a frame made together play together, across
//! switches and when the run falls behind; a source switched off gives silence until switched on (edge 12); and a
//! source lost mid-run is told and gives silence while the run goes on.

use super::*;
use std::sync::Arc;

/// A click's length and level; the rest of a click track is silent.
const CLICK: Duration = Duration::from_millis(1);
const LOUD: f32 = 0.5;

/// The sample `level` makes in the track, within one step of rounding.
fn near(sample: i16, level: f32) -> bool {
    (f32::from(sample) - level * f32::from(i16::MAX)).abs() <= 1.0
}

/// The sound frame `s` seconds into the track.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Test times are a few seconds."
)]
fn frame_at(s: f64) -> usize {
    (s * f64::from(RATE)).round() as usize
}

/// The left sample at `s` seconds into the track.
fn at(log: &Log, s: f64) -> i16 {
    log.sound[frame_at(s) * usize::from(CHANNELS)]
}

/// A source that clicks at each of `times`, in seconds.
fn clicks(times: &'static [f64]) -> Voice {
    Voice::shaped(Arc::new(move |t: Duration| {
        let on = times.iter().any(|&c| t >= secs(c) && t < secs(c) + CLICK);
        if on { LOUD } else { 0.0 }
    }))
}

/// Where each click is in the track, in sound frames: the first frame of each loud run.
fn heard(log: &Log) -> Vec<usize> {
    let left: Vec<i16> = log
        .sound
        .iter()
        .step_by(usize::from(CHANNELS))
        .copied()
        .collect();
    let loud = |i: usize| left.get(i).is_some_and(|s| near(*s, LOUD));
    (0..left.len())
        .filter(|&i| loud(i) && (i == 0 || !loud(i - 1)))
        .collect()
}

#[test]
fn a_switched_off_source_gives_silence_until_switched_on() {
    let temp = Temp::new();
    let orders = vec![
        (secs(1.0), Order::Switch(Switch::Mic, false)),
        (secs(2.0), Order::Switch(Switch::Mic, true)),
        (secs(3.0), Order::Stop),
    ];
    let (_, log) = run(script(), &plan(temp.path()), orders);
    assert_eq!(log.jumps, 0);
    let both = MIC + SOUND;
    for (s, level) in [
        (0.0, both),
        (0.99, both),
        (1.0, SOUND),
        (1.99, SOUND),
        (2.0, both),
        (2.99, both),
    ] {
        assert!(near(at(&log, s), level), "{s} s: {}", at(&log, s));
    }
}

#[test]
fn a_sound_and_a_frame_made_together_play_together_across_switches() {
    let temp = Temp::new();
    let mut s = script();
    s.mic = clicks(&[0.5, 1.1, 1.5]);
    s.sound = clicks(&[0.8, 2.5]);
    let orders = vec![
        (secs(1.0), Order::Switch(Switch::Mic, false)),
        (secs(1.2), Order::Switch(Switch::Mic, true)),
        (secs(1.8), Order::Switch(Switch::Sound, false)),
        (secs(2.2), Order::Switch(Switch::Sound, true)),
        (secs(3.0), Order::Stop),
    ];
    let (_, log) = run(s, &plan(temp.path()), orders);
    // The click at 1.1 s came while the microphone was off, so it is not heard.
    let made = [0.5, 0.8, 1.5, 2.5];
    let found = heard(&log);
    assert_eq!(found.len(), made.len(), "{found:?}");
    for (click, t) in found.iter().zip(made) {
        let slot = log.frames.iter().find(|(_, m)| *m >= secs(t)).unwrap().0;
        let frame = usize::try_from(slot * u64::from(RATE) / u64::from(FPS)).unwrap();
        assert!(
            click.abs_diff(frame) <= 1,
            "{t} s: sound at {click}, frame at slot {slot}"
        );
    }
}

#[test]
fn a_run_that_falls_behind_skips_slots_and_keeps_its_sound_whole_and_in_step() {
    let temp = Temp::new();
    let mut s = script();
    s.frame_cost = slow();
    s.mic = clicks(&[0.5, 1.5, 2.5]);
    s.sound = clicks(&[1.0, 2.0]);
    let (notes, log) = run(s, &plan(temp.path()), vec![(secs(3.0), Order::Stop)]);
    assert!(log.frames.windows(2).any(|w| w[1].0 > w[0].0 + 1));
    let slot = Duration::from_secs(1) / u32::from(FPS);
    for (n, made) in &log.frames {
        let plays = Duration::from_secs(*n) / u32::from(FPS);
        let gap = made.max(&plays).saturating_sub(*made.min(&plays));
        assert!(gap <= slot, "slot {n} plays at {plays:?}, made at {made:?}");
    }
    let made = [0.5, 1.0, 1.5, 2.0, 2.5];
    let found = heard(&log);
    assert_eq!(found.len(), made.len(), "{found:?}");
    for (click, t) in found.iter().zip(made) {
        assert!(click.abs_diff(frame_at(t)) <= 1, "{t} s: sound at {click}");
    }
    assert_eq!(times(&notes), [0, 1000, 2000]);
    let Some(RecordNote::End { length_ms, .. }) = notes.last() else {
        panic!("{notes:?}")
    };
    let sound = u64::try_from(log.sound.len() / usize::from(CHANNELS)).unwrap();
    assert_eq!(sound * MS_S / u64::from(RATE), u64::from(*length_ms));
    assert_eq!(log.jumps, 0);
}

#[test]
fn a_stall_of_minutes_writes_its_sound_a_queue_at_a_time_and_keeps_it_whole() {
    let temp = Temp::new();
    let (mut s, mut p) = (script(), plan(temp.path()));
    let asleep = Duration::from_mins(3);
    s.stall = Some((u64::from(FPS), asleep));
    (p.mic, p.sound, p.max_min) = (false, false, 5);
    let stop = secs(1.0) + asleep + secs(1.0);
    let (notes, log) = run(s, &p, vec![(stop, Order::Stop)]);
    assert!(
        log.widest <= widest(),
        "{} samples in one write",
        log.widest
    );
    let Some(RecordNote::End { length_ms, .. }) = notes.last() else {
        panic!("{notes:?}")
    };
    let sound = u64::try_from(log.sound.len() / usize::from(CHANNELS)).unwrap();
    assert_eq!(sound * MS_S / u64::from(RATE), u64::from(*length_ms));
    assert_eq!(log.jumps, 0);
}

#[test]
fn a_source_lost_mid_run_is_told_and_gives_silence_while_the_run_goes_on() {
    let temp = Temp::new();
    let mut s = script();
    s.mic.fails_at = Some(secs(1.0));
    let p = plan(temp.path());
    let (notes, log) = run(s, &p, vec![(secs(2.0), Order::Stop)]);
    let lost = RecordNote::Lost {
        id: ID,
        switch: Switch::Mic,
    };
    assert_eq!(notes.iter().filter(|n| **n == lost).count(), 1, "{notes:?}");
    assert_eq!(notes.last(), Some(&end(2000, RecordEnd::Stopped)));
    assert!(near(at(&log, 0.5), MIC + SOUND));
    assert!(near(at(&log, 1.5), SOUND));
    assert!(p.file.exists());
}
