//! A Compare run's cloud models (edge 12): once the recording ends, it goes to each ticked one before any local model
//! writes it, as the run hands its speech over only before `again`. Each one's text or fault is its own result, and
//! the run ends once every model ended; its numbers are shown, never kept.

use std::collections::BTreeMap;

use kx_module_api::Secret;
use kx_module_api::ports::{CloudNote, LocalTime, RunStep};

use super::end::line;
use super::{Act, Compare, Line, Stage};

impl Compare {
    /// The send of the recording to every cloud model, which then writes; none with no cloud model.
    pub(super) fn send(&mut self) -> Vec<Act> {
        let (Some(plan), false) = (self.plan.as_ref(), self.remotes.is_empty()) else {
            return Vec::new();
        };
        let (take, lang) = (self.take, plan.lang.clone());
        let to = self.remotes.iter().map(|(r, _)| r.clone()).collect();
        for (_, step) in &mut self.remotes {
            *step = RunStep::Writing;
        }
        vec![Act::Send { take, lang, to }]
    }

    /// A note from cloud model `at` of run `take`, its text `wait_ms` after the send and at wall time `now`; a note of
    /// another run, or of a model that ended, is not heard.
    pub fn net(
        &mut self,
        take: u32,
        at: usize,
        note: CloudNote,
        wait_ms: u32,
        now: LocalTime,
    ) -> Vec<Act> {
        let live = take == self.take && matches!(self.stage, Stage::Running(_));
        let writing = self
            .remotes
            .get(at)
            .is_some_and(|(_, s)| *s == RunStep::Writing);
        if !live || !writing {
            return Vec::new();
        }
        match note {
            CloudNote::Piece { index, text } => {
                self.texts.insert((at, index), text);
                return Vec::new();
            }
            CloudNote::Done => self.score_remote(at, wait_ms),
            CloudNote::Fault(fault) => {
                self.texts.retain(|(of, _), _| *of != at);
                self.remotes[at].1 = RunStep::Failed;
                self.faults.insert(at, (fault, now));
            }
        }
        match self.stage {
            Stage::Running(Some(at)) if at >= self.turns.len() => self.settle(),
            _ => Vec::new(),
        }
    }

    /// Every local model ran: the run ends once no cloud model still writes.
    pub(super) fn settle(&mut self) -> Vec<Act> {
        self.stage = Stage::Running(Some(self.turns.len()));
        let writes = self.remotes.iter().any(|(_, s)| *s == RunStep::Writing);
        if writes { Vec::new() } else { self.finish() }
    }

    /// Cloud model `at`'s result from its texts, which are wiped; its text stays only with no set text.
    fn score_remote(&mut self, at: usize, wait_ms: u32) {
        let (mine, rest) = std::mem::take(&mut self.texts)
            .into_iter()
            .partition(|((of, _), _)| *of == at);
        self.texts = rest;
        let mine: BTreeMap<_, Secret> = mine;
        let (Some(plan), Some((remote, step))) = (self.plan.as_ref(), self.remotes.get_mut(at))
        else {
            return;
        };
        *step = RunStep::Done { wait_ms };
        let marked = line(plan, mine.values(), remote.name.clone(), wait_ms);
        self.lines.push(Line {
            cloud: Some(remote.clone()),
            ..marked
        });
    }
}
