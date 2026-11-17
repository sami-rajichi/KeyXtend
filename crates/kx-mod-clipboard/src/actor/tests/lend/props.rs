//! Property test of the lent paste: random asks, answers right or stale, clock steps, copies, clears, echoes, failed
//! reads and inserts never run a lend beside an insert, answer each ask once, and always end with the clipboard put
//! back, never the text pasted.

use std::time::Duration;

use kx_module_api::ports::{ClipAct, ClipError, ClipNote, PasteText, Pasted};
use kx_module_api::{Clock, Redacted, Zeroizing};
use proptest::prelude::*;

use super::super::super::Msg;
use super::super::super::paste::Paste;
use super::super::ears::Heard;
use super::super::rig::Rig;
use super::{SAID, WAIT_MS};
use crate::fixture::text;
use crate::settings::ClipSettings;

/// Random runs tried; each builds a rig.
const CASES: u32 = 256;
/// The longest run of steps.
const MAX_STEPS: usize = 40;
/// The rounds a drain may take: each clock step ends one stage.
const DRAIN: usize = 8;
/// The share of answers that say the paste went out.
const OK_SHARE: f64 = 0.8;
/// The text copied.
const COPY: &str = "x";

/// One thing that can happen around a lent paste.
#[derive(Clone, Debug)]
enum Step {
    /// Another module asks for its text pasted.
    Lend,
    /// The keyboard answers the last request, or an older one when `back`.
    Answer { back: bool, ok: bool },
    /// Moves the clock on by this many ms.
    Advance(u64),
    /// The user copies `COPY`.
    Copy,
    /// The watch tells a marked copy.
    Marked,
    /// The user clears the clipboard.
    Clear,
    /// The next text read is refused as busy.
    Busy,
    /// Opens the panel, ticks the top row and clicks Insert.
    Insert,
}

/// Any step; the clock steps hit the gap and the paste wait most.
fn step() -> impl Strategy<Value = Step> {
    let s = ClipSettings::from_defaults().unwrap();
    let ms = prop_oneof![
        2 => Just(s.paste_gap_ms),
        2 => Just(s.paste_wait_ms),
        1 => 0..s.paste_wait_ms,
    ];
    prop_oneof![
        4 => Just(Step::Lend),
        4 => (any::<bool>(), prop::bool::weighted(OK_SHARE)).prop_map(|(back, ok)| Step::Answer { back, ok }),
        3 => ms.prop_map(|n| Step::Advance(u64::from(n))),
        2 => Just(Step::Copy),
        2 => Just(Step::Marked),
        1 => Just(Step::Clear),
        1 => Just(Step::Busy),
        1 => Just(Step::Insert),
    ]
}

/// A rig and the asks made so far.
struct Run {
    rig: Rig,
    asks: u32,
}

impl Run {
    /// Does `step`.
    fn apply(&mut self, step: &Step) {
        let rig = &mut self.rig;
        match step {
            Step::Lend => {
                self.asks += 1;
                let text = Redacted::new(Zeroizing::new(SAID.to_owned()));
                let ask = PasteText {
                    round: self.asks,
                    text,
                    wait_ms: WAIT_MS,
                };
                let now = rig.fakes.clock.now();
                rig.send(Msg::Lend(ask, now));
            }
            Step::Answer { back, ok } => {
                let last = rig.pastes().last().map_or(0, |p| p.round);
                let round = last.wrapping_sub(u32::from(*back));
                rig.send(Msg::Pasted(Pasted { round, ok: *ok }));
            }
            Step::Advance(ms) => rig.advance(Duration::from_millis(*ms)),
            Step::Copy => {
                rig.fakes.clip.set_text(Some(COPY));
                rig.copy(vec![text(COPY)], None);
            }
            Step::Marked => rig.push(ClipNote::Marked { app: None }),
            Step::Clear => {
                rig.fakes.clip.set_text(None);
                rig.push(ClipNote::Cleared { app: None });
            }
            Step::Busy => rig.fakes.clip.fail_text(ClipError::Busy),
            Step::Insert => {
                if rig.panels().last() != Some(&true) {
                    rig.open();
                }
                rig.tick_row(0);
                rig.send(Msg::Act(ClipAct::Insert));
            }
        }
    }

    /// The rounds answered, in order.
    fn answers(&self) -> Vec<u32> {
        let pick = |h| match h {
            Heard::Lent(t) => Some(t.round),
            _ => None,
        };
        let mut rounds: Vec<u32> = self.rig.all().into_iter().filter_map(pick).collect();
        rounds.sort_unstable();
        rounds
    }

    /// Does `step`, then checks no lend runs beside an insert and no ask is answered twice.
    fn step(&mut self, step: &Step) -> Result<(), TestCaseError> {
        self.apply(step);
        let inserting = !matches!(self.rig.pasting(), Paste::Idle);
        let idle = self.rig.actor.lent.idle();
        prop_assert!(idle || !inserting, "a lend never runs beside an insert");
        let mut answers = self.answers();
        let all = answers.len();
        answers.dedup();
        prop_assert_eq!(answers.len(), all, "each ask is answered once");
        Ok(())
    }

    /// Waits until nothing is under way; then every ask was answered, and the text pasted is not left.
    fn drain(&mut self) -> Result<(), TestCaseError> {
        let wait = self.rig.settings.paste_wait();
        for _ in 0..DRAIN {
            self.rig.advance(wait);
        }
        prop_assert!(self.rig.actor.lent.idle(), "a lend always ends");
        prop_assert_eq!(
            self.answers(),
            (1..=self.asks).collect::<Vec<_>>(),
            "every ask answered"
        );
        let last = self.rig.fakes.writer.writes().pop();
        prop_assert!(last != Some(vec![text(SAID)]), "the clipboard is put back");
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_lends_run_alone_answer_once_and_always_put_the_clipboard_back(
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut run = Run { rig: Rig::new(), asks: 0 };
        for step in &steps {
            run.step(step)?;
        }
        run.drain()?;
    }
}
