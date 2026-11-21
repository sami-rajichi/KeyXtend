//! A Compare run's turns ending: a model's numbers, a fault that skips a model or ends the run, the run's close, and
//! the finish that keeps the numbers.

use kx_module_api::ports::{BarHint, RunStep, SpeechDone, SpeechError};
use kx_module_api::{Redacted, Secret, Zeroizing};

use super::{Act, Compare, Line, Plan, Stage, Turn};
use crate::counts::{Board, Score};
use crate::flow::{RunCall, mic_hint};
use crate::score::wrong;

impl Compare {
    /// Model `at`'s numbers from its pieces, which are wiped; its text stays only with no set text.
    pub(super) fn score(&mut self, at: usize, done: &SpeechDone) {
        let pieces = std::mem::take(&mut self.pieces);
        let (Some(turn), Some(plan)) = (self.turns.get_mut(at), self.plan.as_ref()) else {
            return;
        };
        turn.step = RunStep::Done {
            wait_ms: done.wait_ms,
        };
        let card = (turn.ask.card.as_ref()).filter(|_| done.card);
        let marked = line(
            plan,
            pieces.values(),
            turn.entrant.name.clone(),
            done.wait_ms,
        );
        self.lines.push(Line {
            print: Some(turn.entrant.ready.print),
            mb: done.memory_mb,
            card: card.map(|c| c.id.clone()),
            ..marked
        });
    }

    /// Ends the run, cancelled first when `cancel`, which drops the cloud uploads; the page goes back to the set text.
    pub(super) fn end(&mut self, cancel: bool) -> Vec<Act> {
        let mut acts = Vec::new();
        if cancel {
            acts.push(self.call(RunCall::Cancel));
        }
        acts.push(self.call(RunCall::Close));
        if cancel && !self.remotes.is_empty() {
            acts.push(Act::Halt(self.take));
        }
        self.texts.clear();
        self.pieces.clear();
        self.stage = Stage::Ready;
        acts
    }

    /// A fault: a card's lets the processor go on; a model's marks it failed and skips it; the recording's ends the
    /// run with its note.
    pub(super) fn fault(&mut self, error: SpeechError) -> Vec<Act> {
        let Stage::Running(Some(at)) = self.stage else {
            if !error.ends() {
                return Vec::new();
            }
            self.hint = Some(mic_hint(error).unwrap_or(BarHint::Fault));
            return self.end(true);
        };
        if !error.ends() {
            return self.fell(at, error);
        }
        if matches!(
            error,
            SpeechError::NoSound | SpeechError::Gone | SpeechError::Timeout
        ) {
            return self.lost();
        }
        self.pieces.clear();
        if let Some(turn) = self.turns.get_mut(at) {
            turn.step = RunStep::Failed;
        }
        self.next(at + 1)
    }

    /// The card of model `at` failed: its check of that file is dropped.
    pub(super) fn fell(&self, at: usize, error: SpeechError) -> Vec<Act> {
        let turn = self.turns.get(at);
        let card = turn.and_then(|t| t.ask.card.as_ref()).map(|c| c.id.clone());
        match (error, card, turn) {
            (SpeechError::CardFailed, Some(card), Some(t)) => vec![Act::Fell {
                card,
                model: t.entrant.ready.print,
            }],
            _ => Vec::new(),
        }
    }

    /// The recording is gone: the run ends with the local models already written or failed, once the cloud ones end.
    pub(super) fn lost(&mut self) -> Vec<Act> {
        let ended = |t: &Turn| matches!(t.step, RunStep::Done { .. } | RunStep::Failed);
        self.turns.retain(ended);
        self.settle()
    }

    /// Every model ran: the run closes and the local models' numbers are kept, if any.
    pub(super) fn finish(&mut self) -> Vec<Act> {
        let mut acts = self.end(false);
        let Some(plan) = self.plan.as_ref().filter(|_| !self.lines.is_empty()) else {
            self.hint.get_or_insert(BarHint::Fault);
            return acts;
        };
        let score = |l: &Line| {
            Some(Score {
                model: l.print?,
                wrong: l.wrong,
                words: l.words,
                wait_ms: l.wait_ms,
                mb: l.mb,
            })
        };
        let board = Board {
            lang: plan.lang.clone(),
            day: plan.day,
            auto: plan.auto,
            card: self.served().cloned(),
            scores: self.lines.iter().filter_map(score).collect(),
        };
        // A cloud model's numbers are shown, never kept, so a run of cloud models alone keeps nothing.
        if !board.scores.is_empty() {
            acts.push(Act::Save(board));
        }
        self.stage = Stage::Results;
        acts
    }
}

/// A model's line from its text's `pieces` in order, joined in a wiped buffer: the set text's words wrong, or the text
/// itself with no set text. Where it ran is left to the caller.
pub(super) fn line<'p>(
    plan: &Plan,
    pieces: impl Iterator<Item = &'p Secret>,
    name: Redacted<String>,
    wait_ms: u32,
) -> Line {
    let text = Zeroizing::new(
        pieces
            .map(|p| p.expose().as_str())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let set = plan.set.expose();
    let count = (!set.is_empty())
        .then(|| wrong(set, &text, &plan.lang))
        .flatten();
    Line {
        name,
        print: None,
        wrong: count.map(|c| c.words),
        words: count.map_or(0, |c| c.total),
        wait_ms,
        mb: 0,
        card: None,
        cloud: None,
        text: count.is_none().then(|| Redacted::new(text)),
    }
}
