//! Paste at once (`insert` = "paste", owner 2026-11-17): a text goes to the clipboard history, which pastes it and puts
//! the clipboard back, and an erase goes to the typer first. A refusal, or no answer within `paste_ms`, types the text
//! and drops a late answer, so it goes in once.

use std::collections::VecDeque;
use std::mem;

use kx_module_api::ports::{ClipText, PasteText, TextAsk, TextDone, TextPasted};
use kx_module_api::{Mono, Redacted, Zeroizing};

use super::Actor;
use super::typing::Source;
use crate::flow::In;

/// A text the clipboard history has not answered: its round, who asked, the text to type instead, and when.
struct Asked {
    round: u32,
    source: Source,
    ask: TextAsk,
    since: Mono,
}

/// The texts out to be pasted, oldest first, and the one waiting for its erase.
#[derive(Default)]
pub(super) struct Pasting {
    last: u32,
    asked: VecDeque<Asked>,
    /// Who asked, and the text pasted once the typer erased what it replaces.
    erasing: Option<(Source, TextAsk)>,
    /// The key page's take waiting for its answer: its round and when it was asked.
    pub(super) take: Option<(u32, Mono)>,
}

impl Actor {
    /// Puts `ask` in at once: its erase is typed first, and an ask with no text is only typed.
    pub(super) fn paste_text(&mut self, source: Source, mut ask: TextAsk) {
        if ask.text.expose().is_empty() {
            return self.type_text(source, ask);
        }
        if ask.erase == 0 {
            return self.ask_paste(source, ask);
        }
        let erase = TextAsk {
            round: ask.round,
            erase: mem::take(&mut ask.erase),
            text: Redacted::new(Zeroizing::new(String::new())),
            gap_ms: ask.gap_ms,
        };
        self.pasting.erasing = Some((source, ask));
        self.type_text(Source::Erase, erase);
    }

    /// The typer erased, or not: the text waiting is pasted, or typed into a password box met meanwhile, else its asker
    /// hears nothing went in.
    pub(super) fn erased(&mut self, ok: bool) {
        let Some((source, ask)) = self.pasting.erasing.take() else {
            return;
        };
        if ok && self.flow.password() {
            return self.type_text(source, ask);
        }
        if ok {
            return self.ask_paste(source, ask);
        }
        if let Source::Flow(round) = source {
            let (count, ok) = (0, false);
            self.step(In::Typed(TextDone { round, count, ok }));
        }
    }

    /// The clipboard history's answer: a pasted text counts as typed in full, one it refused is typed now.
    pub(super) fn pasted(&mut self, done: TextPasted) {
        let asked = &mut self.pasting.asked;
        let Some(asked) =
            (asked.iter().position(|a| a.round == done.round)).and_then(|at| asked.remove(at))
        else {
            return;
        };
        if !done.ok {
            tracing::info!("the clipboard history did not paste a text, so it is typed");
            return self.type_text(asked.source, asked.ask);
        }
        if let Source::Flow(round) = asked.source {
            let count = u32::try_from(asked.ask.text.expose().chars().count()).unwrap_or(u32::MAX);
            self.step(In::Typed(TextDone {
                round,
                count,
                ok: true,
            }));
        }
    }

    /// At `now`, a text whose answer is past `paste_ms` is typed, and a key page's take past `take_ms` shows as busy.
    pub(super) fn paste_tick(&mut self, now: Mono) {
        let s = &self.held.settings;
        let late = |since: Mono, wait| now.saturating_since(since) >= wait;
        let (paste, take) = (s.paste_wait(), s.take_wait());
        while self
            .pasting
            .asked
            .front()
            .is_some_and(|a| late(a.since, paste))
        {
            let Some(asked) = self.pasting.asked.pop_front() else {
                return;
            };
            tracing::warn!("the clipboard history did not answer a paste in time, so it is typed");
            self.type_text(asked.source, asked.ask);
        }
        if let Some((round, _)) = self.pasting.take.filter(|(_, since)| late(*since, take)) {
            self.pasting.take = None;
            tracing::warn!("the clipboard history did not answer the key page's Paste in time");
            let dos = self.panel.pasted(round, ClipText::Busy);
            self.dos(dos);
        }
    }

    /// When the oldest text's answer or the key page's take is due, if one is out.
    pub(super) fn paste_due(&self) -> Option<Mono> {
        let s = &self.held.settings;
        let text = (self.pasting.asked.front()).map(|a| a.since.saturating_add(s.paste_wait()));
        let take = (self.pasting.take).map(|(_, since)| since.saturating_add(s.take_wait()));
        [text, take].into_iter().flatten().min()
    }

    /// Hands `ask`'s text to the clipboard history under a round of its own, noted before the bus hears it.
    fn ask_paste(&mut self, source: Source, ask: TextAsk) {
        let round = self.pasting.last.wrapping_add(1);
        self.pasting.last = round;
        let paste = PasteText {
            round,
            text: ask.text.clone(),
            wait_ms: self.held.settings.paste_ms,
        };
        let since = self.ports.clock.now();
        (self.pasting.asked).push_back(Asked {
            round,
            source,
            ask,
            since,
        });
        tracing::debug!("a text went to the clipboard history to paste");
        self.publish(&paste);
    }
}
