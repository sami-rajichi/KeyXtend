//! Long pieces cut at their quietest stretch until each part fits, texts joined in order, and a text cut to fit a note.

use super::*;
use crate::voice::fake::lib::FakeLib;
use crate::voice::library::{Library, Place};
use std::cell::Cell;
use std::path::Path;

thread_local! {
    /// How many samples this test's thread has weighed.
    static WEIGHED: Cell<usize> = const { Cell::new(0) };
}

/// A sample that counts each time it is weighed.
#[derive(Clone, Copy)]
struct Counted;

impl Sample for Counted {
    fn power(self) -> f32 {
        WEIGHED.with(|n| n.set(n.get() + 1));
        1.0
    }
}

/// `n` loud samples with a quiet stretch of `QUIET_MS` starting at `quiet`.
fn loud_with_quiet(n: usize, quiet: usize) -> Vec<f32> {
    let win = SOUND_HZ * QUIET_MS / MS_PER_S;
    (0..n)
        .map(|k| {
            if (quiet..quiet + win).contains(&k) {
                0.0
            } else {
                0.5
            }
        })
        .collect()
}

/// A list of the one span `r`.
fn one(r: std::ops::Range<usize>) -> Vec<std::ops::Range<usize>> {
    vec![r]
}

#[test]
fn a_piece_that_fits_or_a_model_without_a_limit_is_one_span() {
    let pcm = vec![0.5; 1000];
    assert_eq!(spans(&pcm, Some(1000)), one(0..1000));
    assert_eq!(spans(&pcm, None), one(0..1000));
    assert_eq!(spans::<f32>(&[], Some(10)), one(0..0));
}

#[test]
fn a_long_piece_is_cut_inside_its_quiet_stretch() {
    let n = 30 * SOUND_HZ;
    let quiet = 13 * SOUND_HZ;
    let got = spans(&loud_with_quiet(n, quiet), Some(n - 1));
    let win = SOUND_HZ * QUIET_MS / MS_PER_S;
    assert_eq!(got.len(), 2);
    assert!((quiet..quiet + win).contains(&got[0].end), "{got:?}");
    assert_eq!((got[0].start, got[0].end, got[1].end), (0, got[1].start, n));
}

#[test]
fn every_part_fits_and_the_parts_cover_the_piece_in_order() {
    let pcm = vec![0.5; 100_000];
    for most in [1, 7, 999, 40_000] {
        let got = spans(&pcm, Some(most));
        assert!(
            got.iter().all(|s| s.len() <= most && !s.is_empty()),
            "{most}"
        );
        let mut at = 0;
        for s in &got {
            assert_eq!(s.start, at);
            at = s.end;
        }
        assert_eq!(at, pcm.len());
    }
}

#[test]
fn cutting_a_long_piece_weighs_each_sample_at_most_once() {
    let secs = 60;
    let pcm = vec![Counted; secs * SOUND_HZ];
    let got = spans(&pcm, Some(SOUND_HZ));
    assert!(got.len() >= secs, "one-second parts: {}", got.len());
    let weighed = WEIGHED.with(Cell::get);
    assert!(weighed <= pcm.len(), "{weighed} weighs for {}", pcm.len());
}

#[test]
fn the_parts_texts_join_in_order_in_the_asked_language() {
    let lib = FakeLib::default();
    lib.script().most = Some(10);
    let mut model = lib.load(Path::new("m"), &Place::Cpu, 1).unwrap();
    let said = text(&mut model, &[0.5; 25], "fr-FR", &Stop::default()).unwrap();
    let runs: Vec<(usize, String)> = lib
        .script()
        .runs
        .iter()
        .map(|r| (r.1, r.2.clone()))
        .collect();
    let lens: Vec<usize> = runs.iter().map(|r| r.0).collect();
    assert_eq!(lens.iter().sum::<usize>(), 25);
    assert!(runs.iter().all(|r| r.1 == "fr-FR"));
    let want: Vec<String> = lens.iter().map(|n| format!("{n} fr-FR")).collect();
    assert_eq!(*said, want.join(JOIN));
}

#[test]
fn a_stopped_run_fails_and_a_long_text_is_cut_at_a_character() {
    let lib = FakeLib::default();
    let mut model = lib.load(Path::new("m"), &Place::Cpu, 1).unwrap();
    let stop = Stop::default();
    stop.set();
    assert_eq!(
        text(&mut model, &[0.5; 4], "en", &stop).err(),
        Some(Failed::Stopped)
    );
    let mut long = String::with_capacity(MAX_TEXT_BYTES);
    let room = long.capacity();
    fit(&mut long, "a");
    fit(&mut long, &"é".repeat(MAX_TEXT_BYTES));
    assert_eq!(
        long.len(),
        MAX_TEXT_BYTES - 1,
        "the half character is left out"
    );
    assert!(long.starts_with('a') && long[1..].chars().all(|c| c == 'é'));
    fit(&mut long, "a");
    assert_eq!(long.len(), MAX_TEXT_BYTES);
    fit(&mut long, "b");
    assert_eq!(
        (long.len(), long.capacity()),
        (MAX_TEXT_BYTES, room),
        "it never grew"
    );
}

#[test]
fn a_whole_text_has_its_room_from_the_start() {
    let lib = FakeLib::default();
    let mut model = lib.load(Path::new("m"), &Place::Cpu, 1).unwrap();
    let said = text(&mut model, &[0.5; 4], "en", &Stop::default()).unwrap();
    assert!(said.capacity() >= MAX_TEXT_BYTES);
}
