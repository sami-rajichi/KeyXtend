//! A re-timed copy over the scripted studio: 2× halves the length, frames keep their time, 4× of a very short video
//! still gives a frame and its sound (edge 20), 1× copies the file, and a copy never writes over a file.

use super::*;
use crate::fake::{FakeStudio, Script, Tape};
use crate::record::tests::{Temp, script, widest};
use kx_ipc::MAX_VIDEO_MS;
use std::fs;
use std::path::PathBuf;

/// The tape's sound level.
const LEVEL: i16 = 1_000;

/// The tape's frames a second.
fn fps() -> u64 {
    u64::from(script().tape.shape.fps)
}

/// The stereo samples of `frames` of the tape's frames.
fn samples(frames: u64) -> usize {
    usize::try_from(frames * u64::from(RATE) / fps()).unwrap() * usize::from(CHANNELS)
}

/// A script whose tape holds `frames` frames with their sound.
fn taped(frames: u64) -> Script {
    let mut s = script();
    s.tape = Tape {
        frames,
        sound: frames * u64::from(RATE) / fps(),
        level: LEVEL,
        length_ms: u32::try_from(frames * MS_S / fps()).unwrap(),
        ..s.tape
    };
    s
}

/// Copies a made-up video in `temp` at `times` over `s`; the answer, the studio and the copy's path.
fn copy(temp: &Temp, s: Script, times: u8) -> (Result<u32, RecordFault>, FakeStudio, PathBuf) {
    let (from, to) = (temp.path().join("a.mp4"), temp.path().join("a 2x.mp4"));
    fs::write(&from, b"video").unwrap();
    let (studio, _) = FakeStudio::new(s, Vec::new());
    (speed(&studio, &from, &to, times), studio, to)
}

#[test]
fn two_times_keeps_every_second_frame_and_halves_the_length() {
    let temp = Temp::new();
    let (got, studio, to) = copy(&temp, taped(90), 2);
    assert_eq!(got, Ok(1_500));
    let log = studio.world().log.clone();
    let want: Vec<(u64, u64)> = (0..45).map(|k| (k, k * 2)).collect();
    assert_eq!(log.kept, want);
    assert_eq!(log.sound.len(), samples(45));
    assert!(log.sound.iter().all(|s| *s == LEVEL));
    assert!(log.finished && to.exists());
}

#[test]
fn four_times_of_a_very_short_video_still_gives_a_frame_and_its_sound() {
    let temp = Temp::new();
    let (got, studio, to) = copy(&temp, taped(3), 4);
    assert_eq!(got, Ok(33));
    let log = studio.world().log.clone();
    assert_eq!(log.kept, [(0, 0)]);
    assert_eq!(log.sound.len(), samples(1));
    assert_eq!(log.jumps, 0);
    assert!(log.finished && to.exists());
}

#[test]
fn a_copy_of_a_run_that_skipped_slots_keeps_each_frame_at_its_time() {
    let temp = Temp::new();
    let mut s = taped(90);
    // A run that fell behind: a frame every second slot, its sound whole.
    (s.tape.step, s.tape.sound) = (2, s.tape.sound * 2);
    let (got, studio, _) = copy(&temp, s, 4);
    assert_eq!(got, Ok(1_500));
    let log = studio.world().log.clone();
    let want: Vec<(u64, u64)> = (0..45).map(|k| (k, k * 2)).collect();
    assert_eq!(log.kept, want);
    assert_eq!(log.sound.len(), samples(45));
}

#[test]
fn a_long_video_without_sound_pads_its_copy_a_queue_at_a_time() {
    let temp = Temp::new();
    let mut s = taped(3 * 60 * fps());
    s.tape.sound = 0;
    let (got, studio, _) = copy(&temp, s, 2);
    assert_eq!(got, Ok(90_000));
    let log = studio.world().log.clone();
    assert!(
        log.widest <= widest(),
        "{} samples in one write",
        log.widest
    );
    assert_eq!(log.sound.len(), samples(90 * fps()));
    assert_eq!(log.jumps, 0);
}

#[test]
fn a_frame_timed_past_the_longest_video_is_left_out() {
    let temp = Temp::new();
    let mut s = taped(3);
    // A broken file's frames, timed far apart.
    s.tape.step = u64::MAX / 8;
    let (got, studio, _) = copy(&temp, s, 2);
    assert_eq!(got, Ok(33));
    assert_eq!(studio.world().log.kept, [(0, 0)]);
}

#[test]
fn one_times_copies_the_file_as_it_is() {
    let temp = Temp::new();
    let (got, studio, to) = copy(&temp, taped(90), 1);
    assert_eq!(got, Ok(3_000));
    assert_eq!(fs::read(to).unwrap(), b"video");
    assert!(studio.world().log.kept.is_empty());
}

#[test]
fn a_copy_never_tells_a_length_past_the_longest_video() {
    let temp = Temp::new();
    let mut s = taped(90);
    // Media Foundation reads a video at the cap a few ms longer than it was told.
    s.tape.length_ms = MAX_VIDEO_MS + 17;
    let (got, ..) = copy(&temp, s, 1);
    assert_eq!(got, Ok(MAX_VIDEO_MS));
}

#[test]
fn a_copy_never_writes_over_a_file_or_onto_its_source() {
    let temp = Temp::new();
    let (from, to) = (temp.path().join("a.mp4"), temp.path().join("b.mp4"));
    fs::write(&from, b"video").unwrap();
    fs::write(&to, b"keep").unwrap();
    let (studio, _) = FakeStudio::new(taped(90), Vec::new());
    assert_eq!(speed(&studio, &from, &to, 2), Err(RecordFault::Failed));
    assert_eq!(speed(&studio, &from, &from, 2), Err(RecordFault::Failed));
    let missing = temp.path().join("c.mp4");
    assert_eq!(
        speed(&studio, &missing, &temp.path().join("d.mp4"), 2),
        Err(RecordFault::Failed)
    );
    assert_eq!(fs::read(&to).unwrap(), b"keep");
    assert_eq!(fs::read(&from).unwrap(), b"video");
}

/// A change to the script, and the fault it brings.
type Case = (fn(&mut Script), RecordFault);

#[test]
fn a_copy_that_fails_leaves_nothing_and_says_why() {
    let cases: [Case; 3] = [
        (
            |s| s.ready = Err(RecordFault::NoVideo),
            RecordFault::NoVideo,
        ),
        (|s| s.film = Err(RecordFault::Full), RecordFault::Full),
        (|s| s.tape.frames = 0, RecordFault::Failed),
    ];
    for (change, want) in cases {
        let temp = Temp::new();
        let mut s = taped(90);
        change(&mut s);
        let (got, _, to) = copy(&temp, s, 2);
        assert_eq!(got, Err(want));
        assert!(!to.exists(), "{want:?}");
    }
}

#[test]
fn the_sound_is_thinned_by_averaging_each_run_of_frames() {
    let mut thin = Thin::new(2);
    // Stereo frames [10, -10], [20, -20], then [30, -30], [41, -41] across two pushes.
    assert_eq!(thin.push(&[10, -10, 20, -20, 30, -30]), [15, -15]);
    assert_eq!(thin.push(&[41, -41]), [35, -35]);
}
