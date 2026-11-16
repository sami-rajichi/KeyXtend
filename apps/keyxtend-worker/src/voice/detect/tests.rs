//! The judge as Silero's segmenter: a blip under `min_speech_ms` is never told, speech is told late with its start
//! pending, a dip under `min_silence_ms` keeps it, and an unscored frame counts as speech.

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

mod props;

/// Silero's frame, 32 ms.
const FRAME: usize = 512;
/// Probabilities clearly speech, clearly silence, and between the marks.
const LOUD: f32 = 0.9;
const STILL: f32 = 0.1;
const BETWEEN: f32 = 0.4;

/// What the judge tells for each probability, and what it holds pending after each.
fn judge(probs: &[f32], frame: usize) -> (Vec<bool>, Vec<usize>) {
    let mut j = Judge::new(&SILERO, frame);
    probs.iter().map(|&p| (j.take(p), j.pending())).unzip()
}

fn spans(parts: &[(f32, usize)]) -> Vec<f32> {
    parts.iter().flat_map(|&(p, n)| vec![p; n]).collect()
}

#[test]
fn a_blip_shorter_than_min_speech_is_never_told() {
    let (told, pending) = judge(&spans(&[(LOUD, 7), (STILL, 4)]), FRAME);
    assert!(told.iter().all(|&t| !t));
    assert_eq!(
        pending,
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 0],
        "a dip is pending too"
    );
}

#[test]
fn speech_is_told_from_its_eighth_frame_with_the_seven_before_pending() {
    let (told, pending) = judge(&spans(&[(LOUD, 10)]), FRAME);
    assert_eq!(told, [[false; 7].as_slice(), &[true; 3]].concat());
    assert_eq!(pending[6], 7, "250 ms is under eight frames of 32 ms");
    assert_eq!(pending[7], 0);
}

#[test]
fn a_dip_shorter_than_min_silence_keeps_speech_and_a_longer_one_ends_it() {
    let probs = spans(&[(LOUD, 8), (STILL, 3), (LOUD, 1), (STILL, 4)]);
    let (told, _) = judge(&probs, FRAME);
    assert_eq!(&told[7..12], [true; 5]);
    assert_eq!(&told[12..], [true, true, true, false]);
}

#[test]
fn a_value_between_the_marks_neither_starts_nor_ends_speech_but_a_dip_goes_on_through_it() {
    let (told, _) = judge(&spans(&[(BETWEEN, 20)]), FRAME);
    assert!(told.iter().all(|&t| !t));
    let (told, _) = judge(&spans(&[(LOUD, 8), (BETWEEN, 20)]), FRAME);
    assert!(told[7..].iter().all(|&t| t));
    let (told, _) = judge(&spans(&[(LOUD, 8), (STILL, 1), (BETWEEN, 3)]), FRAME);
    assert_eq!(told.last(), Some(&false));
}

#[test]
fn a_dip_in_the_onset_does_not_count_as_speech_and_a_long_one_drops_it() {
    let (told, _) = judge(&spans(&[(LOUD, 4), (STILL, 2), (LOUD, 4)]), FRAME);
    assert_eq!(
        told.iter().position(|&t| t),
        Some(7),
        "a dip that ends is inside the speech"
    );
    let (told, pending) = judge(&spans(&[(LOUD, 6), (STILL, 4), (LOUD, 2)]), FRAME);
    assert!(told.iter().all(|&t| !t));
    assert_eq!(pending.last(), Some(&2), "a new onset");
}

#[test]
fn the_rules_count_milliseconds_whatever_the_frame() {
    let ten_ms = SOUND_HZ / 100;
    let (told, _) = judge(&spans(&[(LOUD, 26), (STILL, 10)]), ten_ms);
    assert_eq!(told.iter().position(|&t| t), Some(25));
    assert_eq!(told.last(), Some(&false));
    assert_eq!(told.iter().filter(|&&t| t).count(), 10);
}

/// A probe that answers `answer` and keeps every frame it was given.
struct Fake {
    answer: Option<f32>,
    seen: Rc<RefCell<Vec<Vec<f32>>>>,
}

impl Probe for Fake {
    fn frame(&self) -> usize {
        4
    }

    fn prob(&mut self, frame: &[f32]) -> Option<f32> {
        self.seen.borrow_mut().push(frame.to_vec());
        self.answer
    }
}

fn voiced(answer: Option<f32>) -> (Voiced, Rc<RefCell<Vec<Vec<f32>>>>) {
    let seen = Rc::default();
    let probe = Fake {
        answer,
        seen: Rc::clone(&seen),
    };
    (Voiced::new(Box::new(probe), &SILERO), seen)
}

#[test]
fn the_detector_hands_the_probe_floats_and_tells_what_the_judge_decides() {
    let (mut det, seen) = voiced(Some(STILL));
    assert_eq!(det.frame(), 4);
    assert!(!det.speech(&[0, i16::MAX, 0, -i16::MAX]));
    assert_eq!(seen.borrow()[0], [0.0, 1.0, 0.0, -1.0]);
    assert_eq!(det.pending(), 0);
}

#[test]
fn a_frame_the_probe_cannot_score_counts_as_speech_so_no_sound_is_lost() {
    let (mut det, _) = voiced(None);
    let told: Vec<bool> = (0..2000).map(|_| det.speech(&[0; 4])).collect();
    assert!(told.last().copied().unwrap_or(false));
    assert_eq!(det.pending(), 0);
}
