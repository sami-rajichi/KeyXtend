//! Property test of the lent paste: random asks, answers right or stale, clock steps, copies, private, numbered and
//! skipped copies, clears, echoes, failed reads, inserts, takes (one read before its note) and deletes never run a lend
//! beside an insert, answer each ask once, never write a private, skipped or numbered text, never write back a taken or
//! deleted text until it is kept again, and always end with the clipboard put back, empty when its text left the
//! history, unless the watch told a change since. A numbered copy is one marked by number markers alone; it may come
//! back only when taken as our echo.

use kx_module_api::ports::ClipPart;
use proptest::prelude::*;

use super::super::super::paste::Paste;
use super::super::ears::Heard;
use super::super::rig::Rig;
use super::SAID;
use super::skip::skipping;
use crate::entry::text_of;
use crate::fixture::text;
use steps::{NUMBERED, PRIVATE, Step, VAULTED, step};

mod steps;

/// Random runs tried; each builds a rig.
const CASES: u32 = 256;
/// The longest run of steps.
const MAX_STEPS: usize = 40;
/// The rounds a drain may take: each clock step ends one stage.
const DRAIN: usize = 8;

/// Whether the clipboard holds a numbered copy taken as our echo, and whether the lend's text was written over one.
#[derive(Default)]
struct Numbered {
    echoed: bool,
    kept: bool,
}

/// A rig, the asks made so far, whether the watch told a change since the text was last written, whether a skipped
/// app's copy came after every other change, our writes whose echo is due, the numbered copy's flags, whether an early
/// take's note is due, and the texts that left the history, never written back until kept again.
struct Run {
    rig: Rig,
    asks: u32,
    told: bool,
    vault: bool,
    due: u32,
    numbered: Numbered,
    late: bool,
    gone: Vec<String>,
}

impl Run {
    /// A run on a rig that skips `VAULT`'s copies.
    fn new() -> Self {
        Self {
            rig: skipping(),
            asks: 0,
            told: false,
            vault: false,
            due: 0,
            numbered: Numbered::default(),
            late: false,
            gone: Vec::new(),
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

    /// True while an insert runs.
    fn inserting(&self) -> bool {
        !matches!(self.rig.pasting(), Paste::Idle)
    }

    /// The titles of the entries the history holds.
    fn titles(&self) -> Vec<String> {
        let entries = self.rig.actor.history.entries();
        entries.iter().map(|e| e.title.expose().clone()).collect()
    }

    /// Adds the texts this step took out of the history to the gone ones; a text an entry holds again leaves them.
    fn forgot(&mut self, before: &[String]) {
        let now = self.titles();
        let removed = before.iter().filter(|t| !now.contains(t)).cloned();
        self.gone.extend(removed);
        self.gone.retain(|t| !now.contains(t));
    }

    /// Notes what `step` changed, given the writes before it, whether its mark was taken as our echo and whether an
    /// insert ran; a note not ours drops the echoes due, and only an echoed numbered copy or our echo keeps `echoed`.
    fn note(
        &mut self,
        step: &Step,
        writes: usize,
        echo: bool,
        insert: bool,
    ) -> Result<(), TestCaseError> {
        let moved = step.notes();
        self.told |= moved;
        self.vault = matches!(step, Step::Skipped)
            || (self.vault && !moved && !matches!(step, Step::Insert));
        let echoed = &mut self.numbered.echoed;
        *echoed = match step {
            Step::Numbered => echo,
            Step::Marked => *echoed && echo,
            _ => *echoed && !moved,
        };
        let now = self.rig.fakes.writer.writes();
        let wrote = u32::try_from(now.len() - writes).unwrap();
        if moved && !echo {
            self.due = 0;
        }
        self.due = self.due - u32::from(echo) + wrote;
        if wrote > 0 && now.last() == Some(&vec![text(SAID)]) {
            self.told = false;
        }
        self.wrote(&now[writes..], insert)
    }

    /// Follows this step's writes, each then on the clipboard: the lend's text keeps whether it was written over an
    /// echoed numbered copy, only such a copy may come back, and a text gone from the history never does. An `insert`
    /// writes no put-back, and pastes an entry it read before a delete, as `joins.rs` checks.
    fn wrote(&mut self, new: &[Vec<ClipPart>], insert: bool) -> Result<(), TestCaseError> {
        let (flags, clip) = (&mut self.numbered, &self.rig.fakes.clip);
        for parts in new {
            if *parts == [text(SAID)] {
                (flags.kept, flags.echoed) = (flags.echoed, false);
                clip.set_marked(Some(SAID));
                continue;
            }
            let gone = text_of(parts).is_some_and(|t| self.gone.iter().any(|g| g == t));
            let back = gone && !insert;
            prop_assert!(
                !back,
                "a taken or deleted text is never written back until kept again"
            );
            let numbered = *parts == [text(NUMBERED)];
            prop_assert!(
                !numbered || flags.kept,
                "a numbered copy is put back only when it was taken as our echo"
            );
            flags.echoed = numbered;
            clip.set_text(text_of(parts));
        }
        Ok(())
    }

    /// Does `step`; the note an early take left due comes first when `step` has one, as the watch tells in order, and
    /// only that note when `step` is `Late`.
    fn step(&mut self, step: &Step) -> Result<(), TestCaseError> {
        if self.late && step.notes() {
            self.one(&Step::Late)?;
        }
        if matches!(step, Step::Late) {
            return Ok(());
        }
        self.one(step)
    }

    /// Does `step`, noting a change the watch told and the texts gone, then checks no private, skipped or numbered
    /// text was written, no lend runs beside an insert and no ask is answered twice.
    fn one(&mut self, step: &Step) -> Result<(), TestCaseError> {
        let (writes, before) = (self.rig.fakes.writer.writes().len(), self.titles());
        let marks = matches!(step, Step::Private | Step::Numbered | Step::Marked);
        let (echo, inserted) = (marks && self.due > 0, self.inserting());
        self.apply(step);
        let insert = inserted || self.inserting();
        self.note(step, writes, echo, insert)?;
        self.forgot(&before);
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
        let idle = self.rig.actor.lent.idle();
        prop_assert!(
            idle || !self.inserting(),
            "a lend never runs beside an insert"
        );
        let mut answers = self.answers();
        let all = answers.len();
        answers.dedup();
        prop_assert_eq!(answers.len(), all, "each ask is answered once");
        Ok(())
    }

    /// Waits until nothing is under way; then every ask was answered, and the text pasted is not left.
    fn drain(&mut self) -> Result<(), TestCaseError> {
        let wait = self.rig.settings.paste_wait();
        let (writes, insert) = (self.rig.fakes.writer.writes().len(), self.inserting());
        for _ in 0..DRAIN {
            self.rig.advance(wait);
        }
        let now = self.rig.fakes.writer.writes();
        self.wrote(&now[writes..], insert)?;
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
        let mut run = Run::new();
        for step in &steps {
            run.step(step)?;
        }
        run.drain()?;
    }
}
