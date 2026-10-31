//! Inserting (spec §7 "Insert", ADR-0034 decision 2): the files are looked at first, then each step is written to
//! the clipboard and the keyboard is asked to paste it once, waiting for `Pasted` up to `paste_wait_ms`, then
//! `paste_gap_ms`. A missed paste or a files thread gone stops with a note (Ruling 41).

mod step;

use std::mem;

use kx_module_api::Mono;
use kx_module_api::ports::{FileFact, PanelNote, Pasted};

use super::Actor;
use crate::insert::{Insert, Sorted, files_of, sort};
use crate::media::{Look, Why};
use step::Run;

/// Where an insert is.
#[derive(Debug, Default)]
pub(super) enum Paste {
    /// None under way.
    #[default]
    Idle,
    /// Insert `n` waits for its files to be looked at.
    Checking {
        /// The insert's number.
        n: u32,
        /// Its entries, in tick order.
        ids: Vec<u64>,
    },
    /// A step was written and the keyboard asked to paste it.
    Asked {
        /// The steps left.
        run: Run,
        /// The request's round.
        round: u32,
        /// The entry written, none for a join.
        entry: Option<u64>,
        /// When it was asked.
        since: Mono,
    },
    /// The keyboard pasted; the next step waits until `until`.
    Gap {
        /// The steps left.
        run: Run,
        /// When the next step goes.
        until: Mono,
    },
}

/// The insert's state and its counters.
#[derive(Debug, Default)]
pub(super) struct Pasting {
    /// Where it is.
    pub(super) state: Paste,
    /// The last insert's number.
    n: u32,
    /// The last paste request's round.
    round: u32,
}

impl Actor {
    /// Starts inserting `ids` in tick order once their files are looked at; one under way goes on alone.
    pub(super) fn insert(&mut self, ids: Vec<u64>) {
        if !matches!(self.paste.state, Paste::Idle) {
            return;
        }
        self.redraw |= self.note.take().is_some();
        let items = files_of(&ids, &self.history);
        if items.is_empty() {
            return self.step(Run::new(Insert::new(ids, Sorted::default())));
        }
        self.paste.n = self.paste.n.wrapping_add(1);
        let (n, why) = (self.paste.n, Why::Insert(self.paste.n));
        if self.ports.files.send(Look { why, items }).is_err() {
            return self.finish(Some(PanelNote::PasteFailed));
        }
        self.paste.state = Paste::Checking { n, ids };
    }

    /// The files of insert `n` were looked at: entries whose files are all gone leave, those out of reach wait.
    pub(super) fn checked(&mut self, n: u32, items: &[(u64, Vec<FileFact>)]) {
        let waits = matches!(self.paste.state, Paste::Checking { n: at, .. } if at == n);
        if !waits {
            return;
        }
        let Paste::Checking { ids, .. } = mem::take(&mut self.paste.state) else {
            return;
        };
        let sorted = sort(items, &self.history);
        for &id in &sorted.gone {
            self.remove(id);
        }
        self.step(Run::new(Insert::new(ids, sorted)));
    }

    /// Takes the keyboard's answer: a paste that went out moves its entry up and waits the gap; one that did not stops.
    pub(super) fn pasted(&mut self, pasted: Pasted) {
        let asked = matches!(self.paste.state, Paste::Asked { round, .. } if round == pasted.round);
        if !asked {
            return;
        }
        let Paste::Asked { run, entry, .. } = mem::take(&mut self.paste.state) else {
            return;
        };
        if !pasted.ok {
            return self.finish(Some(PanelNote::PasteFailed));
        }
        if let Some(id) = entry {
            self.raise(id);
        }
        let until = self.ports.clock.now();
        let until = until.saturating_add(self.settings.paste_gap());
        self.paste.state = Paste::Gap { run, until };
    }

    /// At `now`: a paste waited for too long stops the insert, and a gap that ended takes the next step.
    pub(super) fn paste_tick(&mut self, now: Mono) {
        if self.paste_due().is_none_or(|due| now < due) {
            return;
        }
        match mem::take(&mut self.paste.state) {
            Paste::Asked { .. } => self.finish(Some(PanelNote::PasteFailed)),
            Paste::Gap { run, .. } => self.step(run),
            other => self.paste.state = other,
        }
    }

    /// When the insert's next timer is due, if one runs.
    pub(super) fn paste_due(&self) -> Option<Mono> {
        match &self.paste.state {
            Paste::Asked { since, .. } => Some(since.saturating_add(self.settings.paste_wait())),
            Paste::Gap { until, .. } => Some(*until),
            Paste::Idle | Paste::Checking { .. } => None,
        }
    }

    /// Ends the insert: a note keeps the panel open to show it, else the panel closes (spec: "the panel then closes").
    fn finish(&mut self, note: Option<PanelNote>) {
        self.paste.state = Paste::Idle;
        match note {
            Some(note) => self.say_note(note),
            None if self.panel.is_open() => {
                self.panel.close();
                self.closed();
            }
            None => {}
        }
    }

    /// Moves pasted entry `id` to the top with the time it was pasted.
    fn raise(&mut self, id: u64) {
        let now = self.ports.calendar.now();
        if let Some(from) = self.history.raise(id, &now) {
            self.moved(id, from);
        }
    }
}
