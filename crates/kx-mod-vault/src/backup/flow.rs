//! The backup pages' state and their keys and clicks (design "The backup box"): the Backup page, the suggested
//! places, the folder list, a password, a box from another computer and the boxes found. ↑↓ move, ←→ move a password's
//! caret, the scroll pad turns pages, Enter presses the highlighted row, and Esc backs out; while a job runs, only Back
//! and Close go.

use kx_module_api::caret::Side;
use kx_module_api::ports::{
    FillButton, FillClick, FillNote, Found, LocalTime, PanelKey, Place, When,
};
use kx_module_api::text::SPACE;

use super::bring::Source;
use super::keepass::{Came, Kdbx};
use super::state::{At, Backup, Flowed};
use crate::entry::Line;
use crate::panel::Step;

/// What a pick of a place or a new password is for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Then {
    /// Setup: the first place, then the password.
    Setup,
    /// Add place.
    Add,
    /// Change password.
    Change,
    /// Bring back: a backup file from the folder list.
    Bring,
    /// Import: a `KeePass` file from the folder list.
    Import,
}

/// What a typed password is for.
#[derive(Clone, Debug)]
pub(crate) enum Goal {
    /// A new password: setup keeps its place for after the padlock; a change has none.
    New { place: Option<Place>, then: Then },
    /// A memory check.
    Check,
    /// A box's password, to bring its entries in.
    Open(Source),
    /// A `KeePass` file's password, to bring its entries in.
    Keepass(Kdbx),
}

/// The Backup page's one question at a time.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Asking {
    /// Remove place, for the place at this index.
    Remove(usize),
    /// Back up now while held: replace the backup, which may hold the entries the notebook lost?
    Replace,
}

/// The backup page that shows.
#[derive(Debug)]
pub(crate) enum Flow {
    /// The Backup page: the highlighted place, and the question that shows.
    Home { at: usize, asking: Option<Asking> },
    /// The suggested places.
    Places {
        at: usize,
        spots: Vec<Place>,
        then: Then,
    },
    /// The folder list: the folders opened, drives first, the rows shown and the highlight.
    Folders {
        trail: Vec<Place>,
        rows: Vec<Place>,
        at: usize,
        then: Then,
    },
    /// A password, typed, and what it is for.
    Password { line: Line, goal: Goal },
    /// A box from another computer in the picked place, and whether to bring its entries in.
    Foreign {
        place: Place,
        when: Option<When>,
        then: Then,
    },
    /// The boxes found, newest first, to bring one back.
    Found { at: usize, rows: Vec<Found> },
}

impl Backup {
    /// True while the backup pages show.
    pub(crate) fn showing(&self) -> bool {
        self.flow.is_some()
    }

    /// Takes a step the panel handed on; a step that changed nothing keeps the note.
    pub(crate) fn step(&mut self, step: &Step, at: &At<'_>) -> Flowed {
        let old = self.note.take();
        let done = match step {
            Step::Open => self.open(at),
            Step::Import => self.import_from(Came::List),
            Step::Click(click) => self.click(*click, at),
            Step::Key(key) => self.key(key, at),
        };
        if done == Flowed::Stay && self.note.is_none() {
            self.note = old;
        }
        done
    }

    /// A click on the shown backup page.
    fn click(&mut self, click: FillClick, at: &At<'_>) -> Flowed {
        let busy = self.asked.is_some();
        match click {
            FillClick::Button(FillButton::Back) => self.back(),
            FillClick::Page { back } => self.turn(back, at.rows),
            _ if busy => Flowed::Stay,
            FillClick::Row { row, .. } => self.row(self.start + usize::from(row), at),
            FillClick::Button(button) => self.button(button, at),
            FillClick::Answer(yes) => self.answer(yes, at),
            FillClick::Checks(on) => self.switch(on),
            FillClick::Caret { at } => self.with_line(|line| line.place_drawn(at.into_inner())),
            _ => Flowed::Stay,
        }
    }

    /// A key on the shown backup page: the password page types, the lists move.
    fn key(&mut self, key: &PanelKey, at: &At<'_>) -> Flowed {
        if *key == PanelKey::Esc {
            return self.esc();
        }
        if self.asked.is_some() {
            return Flowed::Stay;
        }
        match key {
            PanelKey::Up => self.step_by(false),
            PanelKey::Down => self.step_by(true),
            PanelKey::Enter => self.enter(at),
            PanelKey::Left => self.with_line(|line| line.arrow(Side::Left)),
            PanelKey::Right => self.with_line(|line| line.arrow(Side::Right)),
            PanelKey::Delete => self.with_line(Line::delete),
            PanelKey::Esc | PanelKey::Word { .. } => Flowed::Stay,
            PanelKey::Tick | PanelKey::Erase | PanelKey::Text(_) => self.typed(key),
        }
    }

    /// Changes the password line with `change`, which says whether it did.
    fn with_line(&mut self, change: impl FnOnce(&mut Line) -> bool) -> Flowed {
        let Some(Flow::Password { line, .. }) = &mut self.flow else {
            return Flowed::Stay;
        };
        if change(line) {
            Flowed::Show
        } else {
            Flowed::Stay
        }
    }

    /// Types `key` at the password line's caret; a refused character shows the Refused note, and the rest still go in.
    fn typed(&mut self, key: &PanelKey) -> Flowed {
        let Some(Flow::Password { line, .. }) = &mut self.flow else {
            return Flowed::Stay;
        };
        let refused = match key {
            PanelKey::Tick => line.push(SPACE).is_err(),
            PanelKey::Text(text) => text
                .expose()
                .chars()
                .fold(false, |r, c| line.push(c).is_err() | r),
            _ if line.pop().is_some() => false,
            _ => return Flowed::Stay,
        };
        if refused {
            self.note = Some(FillNote::Refused);
        }
        Flowed::Show
    }

    /// Enter: presses the highlighted row, or Done on the password page.
    fn enter(&mut self, at: &At<'_>) -> Flowed {
        match &self.flow {
            Some(Flow::Password { .. }) => self.done_typing(),
            Some(
                Flow::Home { at: i, .. }
                | Flow::Places { at: i, .. }
                | Flow::Folders { at: i, .. }
                | Flow::Found { at: i, .. },
            ) => self.row(*i, at),
            _ => Flowed::Stay,
        }
    }

    /// Esc: a question means No, and every page steps back.
    fn esc(&mut self) -> Flowed {
        match &self.flow {
            Some(
                Flow::Home {
                    asking: Some(_), ..
                }
                | Flow::Foreign { .. },
            ) => self.no(),
            _ => self.back(),
        }
    }

    /// Moves the highlight a row down or up, stopping at the ends.
    fn step_by(&mut self, down: bool) -> Flowed {
        let len = self.len();
        let Some(at) = self.cursor() else {
            return Flowed::Stay;
        };
        let to = if down {
            (*at + 1).min(len.saturating_sub(1))
        } else {
            at.saturating_sub(1)
        };
        let moved = to != *at;
        *at = to;
        if moved { Flowed::Show } else { Flowed::Stay }
    }

    /// Turns to the first row of the page before or after.
    fn turn(&mut self, back: bool, rows: usize) -> Flowed {
        let (len, rows) = (self.len(), rows.max(1));
        let Some(at) = self.cursor() else {
            return Flowed::Stay;
        };
        let (page, end) = (*at / rows, len.saturating_sub(1) / rows);
        let to = if back {
            page.saturating_sub(1)
        } else {
            (page + 1).min(end)
        };
        if to == page {
            return Flowed::Stay;
        }
        *at = to * rows;
        Flowed::Show
    }

    /// How many rows the shown page lists.
    pub(super) fn len(&self) -> usize {
        match &self.flow {
            Some(Flow::Home { .. }) => self.kept.sites.len(),
            Some(Flow::Places { spots, .. }) => spots.len(),
            Some(Flow::Folders { rows, .. }) => rows.len(),
            Some(Flow::Found { rows, .. }) => rows.len(),
            _ => 0,
        }
    }

    /// The shown list's highlight.
    pub(super) fn cursor(&mut self) -> Option<&mut usize> {
        match &mut self.flow {
            Some(
                Flow::Home { at, .. }
                | Flow::Places { at, .. }
                | Flow::Folders { at, .. }
                | Flow::Found { at, .. },
            ) => Some(at),
            _ => None,
        }
    }

    /// True while a password page shows, where letters are typed.
    pub(crate) fn typing(&self) -> bool {
        matches!(self.flow, Some(Flow::Password { .. }))
    }
}

/// When `at` was, in local time, in calendar days before `now`; a time after now, from a clock set back, reads today.
pub(super) fn when(at: crate::entry::Stamp, now: &LocalTime) -> When {
    let local = LocalTime::at(at.unix_ms(), now.offset_min).unwrap_or(*now);
    let days = now.date.days_since(local.date).max(0);
    When {
        days_ago: u32::try_from(days).unwrap_or(u32::MAX),
        date: local.date,
        hour: local.hour,
        minute: local.minute,
    }
}
