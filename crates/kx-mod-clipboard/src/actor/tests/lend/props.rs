//! Property test of the lent paste: random asks, answers right or stale, clock steps, copies, private, numbered and
//! skipped copies, clears, echoes, failed reads and inserts never run a lend beside an insert, answer each ask once,
//! never write a private, skipped or numbered text, and always end with the clipboard put back, unless the watch told a
//! change since. A numbered copy is one marked by number markers alone; it may come back only when an echo was due.

use std::time::Duration;

use kx_module_api::ports::{AppRef, ClipAct, ClipError, ClipNote, PasteText, Pasted};
use kx_module_api::{Clock, Redacted, Zeroizing};
use proptest::prelude::*;

use super::super::super::Msg;
use super::super::super::paste::Paste;
use super::super::ears::Heard;
use super::super::rig::Rig;
use super::skip::{VAULT, skipping};
use super::{SAID, WAIT_MS};
use crate::fixture::{app, text};
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
/// The text a password manager copies with a private marker.
const PRIVATE: &str = "p";
/// The text a skipped app copies, unmarked.
const VAULTED: &str = "v";
/// The text a password manager copies marked by number markers alone, which the presence check lets through.
const NUMBERED: &str = "n";

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
    /// A skipped app copies `VAULTED`.
    Skipped,
    /// A password manager copies `PRIVATE` with a private marker.
    Private,
    /// A password manager copies `NUMBERED`, marked by number markers alone.
    Numbered,
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
        1 => Just(Step::Private),
        1 => Just(Step::Numbered),
        1 => Just(Step::Skipped),
        1 => Just(Step::Clear),
        1 => Just(Step::Busy),
        1 => Just(Step::Insert),
    ]
}

/// A rig, the asks made so far, whether the watch told a change since the text was last written, whether a skipped
/// app's copy came after every other change, our writes whose echo is due, and whether a numbered copy ever came while
/// one was, so it was taken as our echo.
struct Run {
    rig: Rig,
    asks: u32,
    told: bool,
    vault: bool,
    due: u32,
    echoed: bool,
}

impl Run {
    /// The clipboard holds `said`, copied by `by`, and the watch tells it.
    fn copy(&mut self, said: &str, by: Option<AppRef>) {
        self.rig.fakes.clip.set_text(Some(said));
        self.rig.copy(vec![text(said)], by);
    }

    /// Asks for `SAID` pasted on the next round, as the voice module does.
    fn lend(&mut self) {
        self.asks += 1;
        let text = Redacted::new(Zeroizing::new(SAID.to_owned()));
        let ask = PasteText {
            round: self.asks,
            text,
            wait_ms: WAIT_MS,
        };
        let now = self.rig.fakes.clock.now();
        self.rig.send(Msg::Lend(ask, now));
    }

    /// Does `step`.
    fn apply(&mut self, step: &Step) {
        let rig = &mut self.rig;
        match step {
            Step::Lend => self.lend(),
            Step::Answer { back, ok } => {
                let last = rig.pastes().last().map_or(0, |p| p.round);
                let round = last.wrapping_sub(u32::from(*back));
                rig.send(Msg::Pasted(Pasted { round, ok: *ok }));
            }
            Step::Advance(ms) => rig.advance(Duration::from_millis(*ms)),
            Step::Copy => self.copy(COPY, None),
            Step::Skipped => self.copy(VAULTED, Some(app(VAULT))),
            Step::Private => {
                rig.fakes.clip.set_marked(Some(PRIVATE));
                rig.push(ClipNote::Marked { app: None });
            }
            Step::Numbered => {
                rig.fakes.clip.set_text(Some(NUMBERED));
                rig.push(ClipNote::Marked { app: None });
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

    /// Notes what `step` changed, given the writes before it and whether its mark was taken as our echo.
    fn note(&mut self, step: &Step, writes: usize, echo: bool) {
        let moved = matches!(
            step,
            Step::Copy
                | Step::Skipped
                | Step::Private
                | Step::Numbered
                | Step::Marked
                | Step::Clear
        );
        self.told |= moved;
        self.vault = matches!(step, Step::Skipped)
            || (self.vault && !moved && !matches!(step, Step::Insert));
        let now = self.rig.fakes.writer.writes();
        let wrote = u32::try_from(now.len() - writes).unwrap();
        self.due = self.due - u32::from(echo) + wrote;
        self.echoed |= echo && matches!(step, Step::Numbered);
        if wrote > 0 && now.last() == Some(&vec![text(SAID)]) {
            self.told = false;
        }
    }

    /// Does `step`, noting a change the watch told, then checks no private, skipped or numbered text was written, no
    /// lend runs beside an insert and no ask is answered twice.
    fn step(&mut self, step: &Step) -> Result<(), TestCaseError> {
        let writes = self.rig.fakes.writer.writes().len();
        let marks = matches!(step, Step::Private | Step::Numbered | Step::Marked);
        let echo = marks && self.due > 0;
        self.apply(step);
        self.note(step, writes, echo);
        let now = self.rig.fakes.writer.writes();
        prop_assert!(
            !now.contains(&vec![text(PRIVATE)]),
            "a private text is never written"
        );
        let vaulted = now[writes..].contains(&vec![text(VAULTED)]);
        prop_assert!(
            !(self.vault && vaulted),
            "a skipped app's text is never put back"
        );
        prop_assert!(
            self.echoed || !now.contains(&vec![text(NUMBERED)]),
            "a numbered copy is put back only when it was taken as our echo"
        );
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
        let left = last == Some(vec![text(SAID)]);
        prop_assert!(
            !left || self.told,
            "the clipboard is put back unless a change came after"
        );
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn random_lends_run_alone_answer_once_and_always_put_the_clipboard_back(
        steps in prop::collection::vec(step(), 0..MAX_STEPS),
    ) {
        let mut run = Run { rig: skipping(), asks: 0, told: false, vault: false, due: 0, echoed: false };
        for step in &steps {
            run.step(step)?;
        }
        run.drain()?;
    }
}
