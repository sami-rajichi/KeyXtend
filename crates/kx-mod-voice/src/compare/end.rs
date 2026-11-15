//! A Compare run's turns ending: a model's numbers, a fault that skips a model or ends the run, and the end that
//! keeps the numbers.

use kx_module_api::Redacted;
use kx_module_api::Zeroizing;
use kx_module_api::ports::{BarHint, RunStep, SpeechDone, SpeechError};

use super::{Act, Compare, Line, Stage, Turn};
use crate::counts::{Board, Score};
use crate::flow::mic_hint;
use crate::score::wrong;

impl Compare {
    /// Model `at`'s numbers from its pieces, which are wiped; its text stays only with no set text.
    pub(super) fn score(&mut self, at: usize, done: &SpeechDone) {
        let text = Zeroizing::new(
            self.pieces
                .values()
                .map(|p| p.expose().as_str())
                .collect::<Vec<_>>()
                .join(" "),
        );
        self.pieces.clear();
        let (Some(turn), Some(plan)) = (self.turns.get_mut(at), self.plan.as_ref()) else {
            return;
        };
        turn.step = RunStep::Done {
            wait_ms: done.wait_ms,
        };
        let set = plan.set.expose();
        let count = (!set.is_empty())
            .then(|| wrong(set, &text, &plan.lang))
            .flatten();
        let shown = count.is_none().then(|| Redacted::new(text));
        let card = (turn.ask.card.as_ref()).filter(|_| done.card);
        self.lines.push(Line {
            name: turn.entrant.name.clone(),
            print: turn.entrant.ready.print,
            wrong: count.map(|c| c.words),
            words: count.map_or(0, |c| c.total),
            wait_ms: done.wait_ms,
            mb: done.memory_mb,
            card: card.map(|c| c.id.clone()),
            text: shown,
        });
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

    /// The recording is gone: the run ends with the models already written or failed.
    pub(super) fn lost(&mut self) -> Vec<Act> {
        let ended = |t: &Turn| matches!(t.step, RunStep::Done { .. } | RunStep::Failed);
        self.turns.retain(ended);
        self.finish()
    }

    /// Every model ran: the run closes and the numbers are kept, if any.
    pub(super) fn finish(&mut self) -> Vec<Act> {
        let mut acts = self.end(false);
        let Some(plan) = self.plan.as_ref().filter(|_| !self.lines.is_empty()) else {
            self.hint.get_or_insert(BarHint::Fault);
            return acts;
        };
        let score = |l: &Line| Score {
            model: l.print,
            wrong: l.wrong,
            words: l.words,
            wait_ms: l.wait_ms,
            mb: l.mb,
        };
        let board = Board {
            lang: plan.lang.clone(),
            day: plan.day,
            auto: plan.auto,
            card: self.served().cloned(),
            scores: self.lines.iter().map(score).collect(),
        };
        acts.push(Act::Save(board));
        self.stage = Stage::Results;
        acts
    }
}
