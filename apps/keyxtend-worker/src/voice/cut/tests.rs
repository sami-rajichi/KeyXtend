//! The cutter over a detector that hears loud frames: cuts only at a pause once a piece is long enough, keeps the pad,
//! never cuts silence or speech, and, as a property, loses no sample however the sound is chunked.

use super::*;
use crate::voice::fake::{LOUD, Loud};
use proptest::prelude::*;

/// The frame the unit tests' detector judges.
const FRAME: usize = 10;

/// Sound of `(speech, samples)` spans, loud where speech.
fn sound(spans: &[(bool, usize)]) -> Vec<i16> {
    let span = |&(speech, n): &(bool, usize)| vec![if speech { LOUD } else { 0 }; n];
    spans.iter().flat_map(span).collect()
}

/// What the cutter finds in `x`, fed in chunks of `chunk`, with frames of `frame`.
fn cuts_of(x: &[i16], frame: usize, piece: usize, pad: usize, chunks: &[usize]) -> Vec<Cut> {
    let mut cut = Cutter::new(Box::new(Loud(frame)), piece, pad);
    let (mut cuts, mut rest, mut k) = (Vec::new(), x, 0);
    while !rest.is_empty() {
        let (chunk, tail) = rest.split_at(chunks[k % chunks.len()].min(rest.len()));
        cut.take(chunk, &mut cuts).unwrap();
        (rest, k) = (tail, k + 1);
    }
    cut.finish(&mut cuts);
    cuts
}

/// The pieces among `cuts`.
fn pieces(cuts: &[Cut]) -> Vec<Piece> {
    let piece = |c: &Cut| match c {
        Cut::Piece(p) => Some(p.clone()),
        Cut::Heard(_) => None,
    };
    cuts.iter().filter_map(piece).collect()
}

fn piece(start: usize, end: usize, speech: Option<Range<usize>>) -> Piece {
    Piece { start, end, speech }
}

#[test]
fn a_long_speech_is_cut_at_its_pause_keeping_the_pad() {
    let cuts = cuts_of(&sound(&[(true, 150), (false, 50)]), FRAME, 100, 20, &[7]);
    let want = [
        Cut::Heard(true),
        Cut::Heard(false),
        Cut::Piece(piece(0, 170, Some(0..150))),
        Cut::Piece(piece(170, 200, None)),
    ];
    assert_eq!(cuts, want);
}

#[test]
fn no_cut_falls_inside_speech_however_long() {
    let cuts = cuts_of(&sound(&[(true, 500)]), FRAME, 100, 20, &[33]);
    assert_eq!(pieces(&cuts), [piece(0, 500, Some(0..500))]);
}

#[test]
fn silence_alone_is_never_cut_and_counts_no_quiet() {
    let mut cut = Cutter::new(Box::new(Loud(FRAME)), 100, 20);
    let mut cuts = Vec::new();
    cut.take(&sound(&[(false, 1000)]), &mut cuts).unwrap();
    assert_eq!((cuts.len(), cut.quiet()), (0, None));
    cut.finish(&mut cuts);
    assert_eq!(pieces(&cuts), [piece(0, 1000, None)]);
}

#[test]
fn a_short_piece_waits_for_its_length_before_a_pause_cuts_it() {
    let spans = [(true, 30), (false, 30), (true, 30), (false, 100)];
    let cuts = cuts_of(&sound(&spans), FRAME, 100, 20, &[1000]);
    let want = [piece(0, 110, Some(0..90)), piece(110, 190, None)];
    assert_eq!(pieces(&cuts), want);
}

#[test]
fn quiet_counts_the_silence_after_the_last_speech() {
    let mut cut = Cutter::new(Box::new(Loud(FRAME)), 100, 20);
    let mut cuts = Vec::new();
    cut.take(&sound(&[(true, 40), (false, 65)]), &mut cuts)
        .unwrap();
    assert_eq!(cut.quiet(), Some(60), "whole frames only");
}

#[test]
fn a_trim_keeps_the_pad_within_its_piece() {
    let p = piece(100, 300, Some(150..200));
    assert_eq!(
        (p.trim(20), p.trim(80), p.trim(0)),
        (Some(130..220), Some(100..280), Some(150..200))
    );
    assert_eq!(piece(0, 9, None).trim(5), None);
}

/// Each sample's kind: its frame's, and a cut-off tail takes the last frame's.
fn kinds(frames: &[bool], frame: usize, tail: usize) -> Vec<bool> {
    let last = frames.last().copied().unwrap_or(false);
    let body = frames.iter().flat_map(|&f| vec![f; frame]);
    body.chain(std::iter::repeat_n(last, tail)).collect()
}

/// Checks one piece against the kinds: its speech span, and for a piece cut before the end, the rules of a cut.
fn check(
    p: &Piece,
    kinds: &[bool],
    piece_len: usize,
    pad: usize,
    last: bool,
) -> Result<(), TestCaseError> {
    let inside = &kinds[p.start..p.end];
    let first = inside.iter().position(|&k| k).map(|i| i + p.start);
    let end = inside.iter().rposition(|&k| k).map(|i| i + p.start + 1);
    prop_assert_eq!(&p.speech, &first.zip(end).map(|(a, b)| a..b));
    if !last {
        prop_assert!(p.end - p.start >= piece_len && p.speech.is_some());
        prop_assert!(
            kinds[p.end - pad..p.end].iter().all(|&k| !k),
            "the pad is silent"
        );
        prop_assert!(!(kinds[p.end - 1] && kinds[p.end]), "no cut inside speech");
    }
    Ok(())
}

proptest! {
    #[test]
    fn no_sample_is_lost_and_each_cut_follows_the_rules(
        frames in prop::collection::vec(prop::bool::weighted(0.6), 0..120),
        frame in 1_usize..24,
        tail_seed in 0_usize..24,
        piece_len in 1_usize..400,
        pad in 0_usize..120,
        chunks in prop::collection::vec(1_usize..90, 1..6),
    ) {
        let tail = tail_seed % frame;
        let kinds = kinds(&frames, frame, tail);
        let x: Vec<i16> = kinds.iter().map(|&k| if k { LOUD } else { 0 }).collect();
        let found = pieces(&cuts_of(&x, frame, piece_len, pad, &chunks));
        prop_assert_eq!(&found, &pieces(&cuts_of(&x, frame, piece_len, pad, &[usize::MAX])));
        let mut at = 0;
        for (i, p) in found.iter().enumerate() {
            prop_assert_eq!(p.start, at);
            check(p, &kinds, piece_len, pad, i + 1 == found.len())?;
            at = p.end;
        }
        prop_assert_eq!(at, x.len());
    }
}

/// Hears loud frames but tells speech only from the `need`-th loud frame in a row, holding the earlier ones pending.
struct Late {
    loud: Loud,
    need: usize,
    run: usize,
}

impl Late {
    fn boxed(need: usize) -> Box<dyn Detector> {
        let loud = Loud(FRAME);
        Box::new(Self { loud, need, run: 0 })
    }
}

impl Detector for Late {
    fn frame(&self) -> usize {
        self.loud.0
    }

    fn speech(&mut self, frame: &[i16]) -> bool {
        self.run = if self.loud.speech(frame) {
            self.run + 1
        } else {
            0
        };
        self.run >= self.need
    }

    fn pending(&self) -> usize {
        if self.run < self.need { self.run } else { 0 }
    }
}

/// The cuts a `Late` detector finds in `x`, fed whole.
fn late_cuts(x: &[i16], need: usize, piece: usize, pad: usize) -> Vec<Cut> {
    let mut cut = Cutter::new(Late::boxed(need), piece, pad);
    let mut cuts = Vec::new();
    cut.take(x, &mut cuts).unwrap();
    cut.finish(&mut cuts);
    cuts
}

#[test]
fn speech_told_late_starts_where_the_detector_first_held_it() {
    let x = sound(&[(false, 50), (true, 60), (false, 200)]);
    let want = [piece(0, 130, Some(50..110)), piece(130, 310, None)];
    assert_eq!(pieces(&late_cuts(&x, 3, 10, 20)), want);
}

#[test]
fn no_cut_falls_inside_frames_the_detector_holds_pending() {
    let x = sound(&[(true, 100), (false, 40), (true, 50), (false, 100)]);
    let want = [piece(0, 210, Some(0..190)), piece(210, 290, None)];
    assert_eq!(pieces(&late_cuts(&x, 3, 150, 20)), want);
}
