//! What a case saw, and the checks each step's effects and the state after it must pass.

use super::*;
use crate::counts::Board;
use std::collections::BTreeSet;

/// What the case saw so far.
#[derive(Default)]
pub(super) struct Seen {
    opened: BTreeSet<u32>,
    closed: BTreeSet<u32>,
    saved: BTreeSet<u32>,
    sent: BTreeSet<u32>,
    halted: BTreeSet<u32>,
    /// The last run opened.
    pub(super) last: u32,
    /// The last run's set text was empty.
    bare: bool,
    /// The last run's microphone said it opened.
    pub(super) mic: bool,
}

impl Seen {
    /// Checks one step's effects: one Open per run, calls only on the open run, Level only while recording.
    pub(super) fn effects(&mut self, acts: &[Act], compare: &Compare) -> Result<(), TestCaseError> {
        for a in acts {
            match a {
                Act::Open { take, ask } => {
                    prop_assert!(
                        *take != self.last && self.opened.insert(*take),
                        "{take} reopened"
                    );
                    prop_assert!(ask.model.is_none(), "the recording loads no model");
                    let set = compare.plan().map(|p| p.set.expose().is_empty());
                    (self.last, self.bare) = (*take, set.unwrap_or(false));
                    self.mic = false;
                }
                Act::Run { take, call } => {
                    let live = *take == self.last && !self.closed.contains(take);
                    prop_assert!(live, "{call:?} on run {take}");
                    if *call == RunCall::Close {
                        self.closed.insert(*take);
                    }
                }
                Act::Level(_) => prop_assert!(compare.recording(), "a level while not recording"),
                Act::Save(board) => self.saved(board, compare)?,
                Act::Fell { .. } => {}
                Act::Send { take, to, .. } => self.sent(*take, to, compare)?,
                Act::Halt(take) => {
                    prop_assert_eq!(*take, self.last, "an older run halted");
                    self.halted.insert(*take);
                }
            }
        }
        Ok(())
    }

    /// A save: once a run, with the local models' numbers alone.
    fn saved(&mut self, board: &Board, compare: &Compare) -> Result<(), TestCaseError> {
        let last = self.last;
        prop_assert!(self.saved.insert(last), "run {last} saved twice");
        let local = compare.results().iter().filter(|l| l.print.is_some());
        prop_assert_eq!(
            board.scores.len(),
            local.count(),
            "a cloud model's numbers kept"
        );
        Ok(())
    }

    /// A send: once, to exactly the ticked cloud models, of the live run after its recording ended, never after a Halt.
    fn sent(&mut self, take: u32, to: &[Remote], compare: &Compare) -> Result<(), TestCaseError> {
        let live = take == self.last && self.opened.contains(&take) && !self.closed.contains(&take);
        prop_assert!(
            live && !compare.recording(),
            "a send outside a stopped live run"
        );
        prop_assert!(
            !self.halted.contains(&take),
            "run {take} sent after its halt"
        );
        prop_assert!(self.sent.insert(take), "run {take} sent twice");
        let ticked = compare.plan().map(|p| p.clouds.as_slice());
        prop_assert_eq!(Some(to), ticked, "only the ticked cloud models");
        prop_assert!(!to.is_empty(), "a send to no model");
        Ok(())
    }

    /// Checks the state after a step: the lend follows the stage and a resting Compare holds no open run.
    pub(super) fn state(&self, compare: &Compare) -> Result<(), TestCaseError> {
        let lend = compare.lend();
        prop_assert_eq!(compare.busy(), lend != Lend::Off);
        prop_assert_eq!(
            compare.recording(),
            matches!(lend, Lend::Prep | Lend::Listen)
        );
        if compare.recording() {
            prop_assert_eq!(
                lend == Lend::Listen,
                self.mic,
                "Listening only once its microphone opened"
            );
        }
        prop_assert_eq!(compare.running().is_some(), lend == Lend::Write);
        let open = self.opened.len() - self.closed.len();
        prop_assert_eq!(
            open,
            usize::from(compare.busy()),
            "an open run without a busy Compare"
        );
        let models = compare
            .plan()
            .map_or(0, |p| p.entrants.len() + p.clouds.len());
        let results = compare.results();
        prop_assert!(results.is_empty() || !compare.busy(), "results while busy");
        prop_assert!(results.len() <= models, "more results than models");
        let texts = results.iter().all(|l| l.text.is_some() == self.bare);
        prop_assert!(texts, "a text shown with a set text, or none without");
        Ok(())
    }
}
