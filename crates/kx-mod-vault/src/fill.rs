//! Filling (design "Filling", ADR-0033): the panel's fill request becomes one `FillType` for the keyboard, the only
//! typist, with a new round and the Tab wait. Values travel as `FillText`, and only the round and the entry are kept;
//! while a request waits for its answer a new fill is refused, never queued.

use std::time::Duration;

use kx_module_api::ports::{FillNote, FillPart, FillText, FillType, FillTyped};

use crate::book::Notebook;
use crate::entry::Entry;
use crate::panel::{Fill, Part};

/// The requests sent and the one whose answer counts.
#[derive(Debug, Default)]
pub struct Filling {
    /// The last request's round.
    round: u32,
    /// The last request's entry, until its answer comes.
    id: Option<u64>,
    /// The last request still waits, so a new fill is refused.
    waiting: bool,
}

impl Filling {
    /// The request typing `fill` from `book`, with `gap` after each Tab.
    ///
    /// # Errors
    /// `NotTyped` while a request waits, and `Refused` for a value `FillText` refuses; nothing is sent then.
    pub fn ask(
        &mut self,
        fill: &Fill,
        book: &Notebook,
        gap: Duration,
    ) -> Result<FillType, FillNote> {
        if self.waiting {
            return Err(FillNote::NotTyped);
        }
        let entry = book.get(fill.id).ok_or(FillNote::Refused)?;
        let parts = fill.parts.iter().map(|&p| part(entry, p));
        let parts = parts.collect::<Option<Vec<_>>>().ok_or(FillNote::Refused)?;
        self.round = self.round.wrapping_add(1);
        (self.id, self.waiting) = (Some(fill.id), true);
        Ok(FillType {
            round: self.round,
            parts,
            tab_gap: gap,
        })
    }

    /// The keyboard's answer to the last request: its entry when every part went out, else the not-finished note;
    /// `None` for an older round or one answered already.
    pub fn answer(&mut self, typed: FillTyped) -> Option<Result<u64, FillNote>> {
        if typed.round != self.round {
            return None;
        }
        let id = self.id.take()?;
        self.waiting = false;
        Some(if typed.ok {
            Ok(id)
        } else {
            Err(FillNote::NotTyped)
        })
    }

    /// Stops waiting, as the panel closes, so a keyboard that never answers blocks no later fill; until a new fill is
    /// sent, the last request's answer still counts.
    pub fn release(&mut self) {
        self.waiting = false;
    }

    /// True while a fill waits for the keyboard's answer.
    #[must_use]
    pub const fn waiting(&self) -> bool {
        self.waiting
    }
}

/// `part` of a fill of `entry`; `None` for a value `FillText` refuses or a field the entry lacks.
fn part(entry: &Entry, part: Part) -> Option<FillPart> {
    match part {
        Part::Tab => Some(FillPart::Tab),
        Part::Value(i) => {
            let value = entry.fields().get(i)?.value.clone();
            FillText::new(value).map(FillPart::Text)
        }
    }
}

#[cfg(test)]
mod tests;
