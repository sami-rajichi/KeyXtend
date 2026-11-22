//! Paste at once for another module (owner, 2026-11-17, ADR-0048): the history keeps what the clipboard holds, writes
//! the text as ours with the presence markers, asks the keyboard for one paste on a round of its own, answers right
//! after the keyboard does, and puts the clipboard back after `paste_gap_ms` unless another app changed it meanwhile;
//! an entry deleted or a text that left the history meanwhile is put back as an empty clipboard. It answers no, writing
//! nothing, while an insert or a lend runs, past the asker's start, or when `previous` refuses.

use std::mem;
use std::time::Duration;

use kx_module_api::Mono;
use kx_module_api::ports::{ClipError, ClipPart, PasteAsk, PasteText, Pasted, TextPasted};

use super::Actor;
use super::board::{Back, Held, own};
use super::paste::Paste;
use super::take::wipe;
use crate::entry::text_of;

/// Where a lent paste is; `back` is `None` once another app changed the clipboard.
#[derive(Debug, Default)]
pub(super) enum Lend {
    /// None under way.
    #[default]
    Idle,
    /// The text was written and the keyboard asked to paste it on `round`, for the asker's `asker`.
    Asked {
        round: u32,
        asker: u32,
        back: Option<Back>,
        since: Mono,
    },
    /// The keyboard answered; the clipboard goes back at `until`.
    Gap { back: Option<Back>, until: Mono },
}

impl Lend {
    /// True when none is under way.
    pub(super) const fn idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Entry `id` left the history: a put-back due to write it whole writes an empty clipboard instead.
    pub(super) fn gone(&mut self, id: u64) {
        if let Self::Asked { back, .. } | Self::Gap { back, .. } = self
            && let Some(back) = back.as_mut().filter(|b| b.held == Held::Entry(id))
        {
            wipe(mem::take(&mut back.parts));
            back.held = Held::Empty;
        }
    }
}

impl Actor {
    /// Pastes `ask`'s text, asked at `at`, unless an insert or a paste is under way, the paste could not start within
    /// the asker's `start_ms`, or `previous` finds nothing it may put back.
    pub(super) fn lend(&mut self, ask: PasteText, at: Mono) {
        let (asker, start) = (ask.round, Duration::from_millis(ask.start_ms().into()));
        let free = self.lent.idle() && matches!(self.paste.state, Paste::Idle);
        let Some(back) = (free && self.in_time(at, start))
            .then(|| self.previous())
            .flatten()
        else {
            return self.answer(asker, false);
        };
        let parts = vec![ClipPart::Text(own(ask.text))];
        // The presence markers keep other clipboard tools from the text; it is never written back, so none is lost.
        let wrote = self.ports.writer.write_lent(&parts, self.settings.wait());
        wipe(parts);
        if let Err(error) = wrote {
            tracing::warn!(%error, "a text to paste was not written to the clipboard");
            // A held clipboard was left as it was; any other failure may have emptied it.
            if error == ClipError::Busy {
                wipe(back.parts);
            } else {
                self.put_back(Some(back));
            }
            return self.answer(asker, false);
        }
        self.board.wrote(Held::Other);
        if !self.in_time(at, start) {
            self.put_back(Some(back));
            return self.answer(asker, false);
        }
        let (round, since) = (self.paste.next_round(), self.ports.clock.now());
        let back = Some(back);
        self.lent = Lend::Asked {
            round,
            asker,
            back,
            since,
        };
        // The keyboard answers on this thread while it handles this; the answer waits in the inbox, no lock held.
        self.publish(&PasteAsk { round });
    }

    /// The keyboard's answer to the lent paste: the asker hears it at once, and the clipboard goes back after the gap,
    /// as even a paste not all sent may still be read. False when the answer is not the lend's.
    pub(super) fn lend_pasted(&mut self, pasted: Pasted) -> bool {
        let ours = matches!(self.lent, Lend::Asked { round, .. } if round == pasted.round);
        if !ours {
            return false;
        }
        if let Lend::Asked { asker, back, .. } = mem::take(&mut self.lent) {
            self.answer(asker, pasted.ok);
            let until = self
                .ports
                .clock
                .now()
                .saturating_add(self.settings.paste_gap());
            self.lent = Lend::Gap { back, until };
        }
        true
    }

    /// Another app changed the clipboard while a paste is lent: its copy stays, so nothing is put back.
    pub(super) fn lend_moved(&mut self) {
        if let Lend::Asked { back, .. } | Lend::Gap { back, .. } = &mut self.lent
            && let Some(back) = back.take()
        {
            tracing::info!("the clipboard changed during a lent paste, so it is not put back");
            wipe(back.parts);
        }
    }

    /// At `now`: a paste the keyboard never answered puts the clipboard back and answers no, and a gap that ended puts
    /// it back.
    pub(super) fn lend_tick(&mut self, now: Mono) {
        if self.lend_due().is_none_or(|due| now < due) {
            return;
        }
        match mem::take(&mut self.lent) {
            Lend::Asked { asker, back, .. } => {
                tracing::warn!("the keyboard did not answer a lent paste in time");
                self.put_back(back);
                self.answer(asker, false);
            }
            Lend::Gap { back, .. } => self.put_back(back),
            Lend::Idle => {}
        }
    }

    /// When the lent paste's timer is due, if one runs.
    pub(super) fn lend_due(&self) -> Option<Mono> {
        match &self.lent {
            Lend::Asked { since, .. } => Some(since.saturating_add(self.settings.paste_wait())),
            Lend::Gap { until, .. } => Some(*until),
            Lend::Idle => None,
        }
    }

    /// At the stop, the clipboard goes back; an asker still waiting types its text once its own wait ends.
    pub(super) fn lend_quit(&mut self) {
        if let Lend::Asked { back, .. } | Lend::Gap { back, .. } = mem::take(&mut self.lent) {
            self.put_back(back);
        }
    }

    /// True while a paste asked at `at` may still start.
    fn in_time(&self, at: Mono, start: Duration) -> bool {
        self.ports.clock.now().saturating_since(at) <= start
    }

    /// Writes `back` as ours, if another app left the clipboard alone, or an empty clipboard if its text left the history
    /// meanwhile; a failure is logged, as the text pasted then stays.
    fn put_back(&mut self, back: Option<Back>) {
        let Some(mut back) = back else { return };
        if text_of(&back.parts).is_some_and(|t| self.lost.has(t)) {
            wipe(mem::take(&mut back.parts));
            back.held = Held::Empty;
        }
        match self.ports.writer.write(&back.parts, self.settings.wait()) {
            Ok(()) => self.board.wrote(back.held),
            Err(error) => tracing::warn!(%error, "the clipboard was not put back after a paste"),
        }
        wipe(back.parts);
    }

    /// Tells the asker of round `round` whether its text was pasted.
    fn answer(&self, round: u32, ok: bool) {
        self.publish(&TextPasted { round, ok });
    }
}
