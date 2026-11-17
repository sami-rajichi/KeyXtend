//! Any spans of speech probabilities, unscored frames among them, through the detector with any tuning and frame:
//! speech is told only after `min_speech_ms` of onset held pending, ends only after `min_silence_ms` under the lower
//! mark, a quiet frame keeps silence quiet, nothing is pending once told, and the pending frames stay bounded.

use super::*;
use proptest::collection::vec;
use proptest::prelude::*;
use std::cell::Cell;

/// The longest frame a case judges, in samples.
const MAX_FRAME: usize = 1024;
/// The longest `min_speech_ms` and `min_silence_ms` a case tunes.
const MAX_SPEECH_MS: u32 = 600;
const MAX_SILENCE_MS: u32 = 400;
/// The most spans, and frames in one span.
const MAX_SPANS: usize = 20;
const MAX_SPAN: usize = 30;
/// How often a span's frames cannot be scored.
const UNSCORED_SHARE: u32 = 1;
const SCORED_SHARE: u32 = 9;

/// A probe that answers its script in order and counts each frame not `frame` samples long.
struct Script {
    frame: usize,
    probs: Vec<Option<f32>>,
    at: usize,
    wrong: Rc<Cell<usize>>,
}

impl Probe for Script {
    fn frame(&self) -> usize {
        self.frame
    }

    fn prob(&mut self, frame: &[f32]) -> Option<f32> {
        if frame.len() != self.frame {
            self.wrong.set(self.wrong.get() + 1);
        }
        let p = self.probs.get(self.at).copied().flatten();
        self.at += 1;
        p
    }
}

/// A tuning's threshold, fall, floor, `min_silence_ms` and `min_speech_ms`, as `Tune` prints nothing.
type Knobs = (f32, f32, f32, u32, u32);

fn knobs() -> impl Strategy<Value = Knobs> {
    let ms = (0..=MAX_SILENCE_MS, 0..=MAX_SPEECH_MS);
    let marks = (0.05_f32..0.95, 0.0_f32..0.5, 0.0_f32..0.3);
    (marks, ms).prop_map(|((t, fall, floor), (s, p))| (t, fall, floor, s, p))
}

fn tune((threshold, fall, floor, min_silence_ms, min_speech_ms): Knobs) -> Tune {
    Tune {
        threshold,
        fall,
        floor,
        min_silence_ms,
        min_speech_ms,
    }
}

fn probs() -> impl Strategy<Value = Vec<Option<f32>>> {
    let p = prop_oneof![
        UNSCORED_SHARE => Just(None),
        SCORED_SHARE => (0.0_f32..=1.0).prop_map(Some),
    ];
    let spans = vec((p, 1..=MAX_SPAN), 0..MAX_SPANS);
    spans.prop_map(|s| s.into_iter().flat_map(|(p, n)| vec![p; n]).collect())
}

/// Frame `i` as judged, after the frame before was told `was` with `held` pending.
struct Judged {
    i: usize,
    was: bool,
    held: usize,
    told: bool,
    pending: usize,
}

/// Checks frame `f` of probabilities `p`: what stays pending, and how speech starts, ends and stays quiet.
fn check(j: &Judge, p: &[f32], f: &Judged) -> Result<(), TestCaseError> {
    prop_assert!(!f.told || f.pending == 0, "pending while told at {}", f.i);
    prop_assert!(
        f.pending < j.speech + j.silence,
        "too much pending at {}",
        f.i
    );
    if f.told && !f.was {
        prop_assert!(f.held + 1 >= j.speech, "told early at {}", f.i);
    }
    if !f.was && f.held == 0 && p[f.i] < j.up {
        prop_assert!(
            !f.told && f.pending == 0,
            "a quiet frame started speech at {}",
            f.i
        );
    }
    if f.was && !f.told {
        ended(j, p, f.i)?;
    }
    Ok(())
}

/// Speech ended at frame `i`: the `silence` frames up to it stayed under `up`, the first under `down`.
fn ended(j: &Judge, p: &[f32], i: usize) -> Result<(), TestCaseError> {
    let from = (i + 1).checked_sub(j.silence);
    let dip = from.map(|f| &p[f..=i]);
    prop_assert!(
        dip.is_some_and(|d| d.iter().all(|&x| x < j.up)),
        "ended early at {}",
        i
    );
    prop_assert!(
        from.is_some_and(|f| p[f] < j.down),
        "ended over the lower mark at {}",
        i
    );
    Ok(())
}

proptest! {
    #[test]
    fn speech_starts_and_ends_by_the_tuning_whatever_the_frame(
        knobs in knobs(),
        frame in 1..=MAX_FRAME,
        probs in probs(),
    ) {
        let wrong = Rc::new(Cell::new(0));
        let script = Script { frame, probs: probs.clone(), at: 0, wrong: Rc::clone(&wrong) };
        let mut det = Voiced::new(Box::new(script), &tune(knobs));
        let silent = vec![0_i16; det.frame()];
        let (told, pending): (Vec<bool>, Vec<usize>) =
            probs.iter().map(|_| (det.speech(&silent), det.pending())).unzip();
        prop_assert_eq!(wrong.get(), 0, "the probe was given a frame of the wrong length");
        let p: Vec<f32> = probs.iter().map(|x| x.unwrap_or(UNSCORED)).collect();
        for i in 0..p.len() {
            let (was, held) = if i == 0 { (false, 0) } else { (told[i - 1], pending[i - 1]) };
            let f = Judged { i, was, held, told: told[i], pending: pending[i] };
            check(&det.judge, &p, &f)?;
        }
    }
}
