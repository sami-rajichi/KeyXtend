//! Values put into a New or Edit field from outside the keyboard: Make one's password, shown so the user sees it, and
//! Paste's text under the field rules (`field_chars`), into the field Paste was asked on.

use kx_module_api::ports::{ClipText, FillNote, PageNote};

use super::edit::{Editor, Sub};
use super::{Do, Page, Panel};
use crate::entry::{EntryError, Line, text};
use crate::paste::flat;

impl Panel {
    /// Takes what Paste found for the field it was asked on: a text replaces its value, and anything else shows its
    /// note. A form left meanwhile takes nothing.
    pub fn pasted(&mut self, found: &ClipText) -> Do {
        let Page::Edit(edit) = &mut self.page else {
            return Do::Stay;
        };
        let Some(i) = edit.pasting.take() else {
            return Do::Stay;
        };
        let note = match found {
            ClipText::Text(text) => edit.pasted(i, text.expose()),
            ClipText::Empty => Some(FillNote::NoText),
            ClipText::TooLong => Some(FillNote::Refused),
            ClipText::Busy => Some(FillNote::ClipBusy),
        };
        self.note = note.map(PageNote::plain);
        Do::Show
    }

    /// Paste heard no answer in time, so the note says the clipboard history is not on; a late answer is dropped.
    pub fn paste_off(&mut self) -> Do {
        let Page::Edit(edit) = &mut self.page else {
            return Do::Stay;
        };
        if edit.pasting.take().is_none() {
            return Do::Stay;
        }
        self.note = Some(PageNote::plain(FillNote::ClipOff));
        Do::Show
    }
}

impl Editor {
    /// Puts the password `made` into field `i`, shown so the user sees it; false when it does not fit.
    pub(crate) fn made(&mut self, i: usize, made: &str) -> bool {
        let Ok(value) = Line::with(made, self.max) else {
            return false;
        };
        let Some(slot) = self.slots.get_mut(i) else {
            return false;
        };
        slot.value = value;
        self.shown.insert(i);
        self.focus(i, false)
    }

    /// Puts `text` into field `i` in place of its value, flattened for the field, and the form highlights it; the note
    /// says why not, and the field keeps its value then.
    fn pasted(&mut self, i: usize, text: &str) -> Option<FillNote> {
        let (max, form) = (self.max, self.sub == Sub::Form);
        let slot = self.slots.get_mut(i)?;
        let refused = |e: EntryError| e.note().unwrap_or(FillNote::Refused);
        let value = flat(text, slot.hidden).and_then(|mut text| {
            text::fit(&mut text, slot.hidden, max).map_err(refused)?;
            Line::with(&text, max).map_err(refused)
        });
        match value {
            Ok(value) => slot.value = value,
            Err(note) => return Some(note),
        }
        if form {
            self.focus(i, false);
        }
        None
    }
}
