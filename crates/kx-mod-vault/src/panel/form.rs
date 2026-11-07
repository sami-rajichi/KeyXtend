//! The New and Edit form's buttons (design "New and Edit"): the kind chips, the name, Add a field, Your own label,
//! Save, and Delete entry's question with its answer; and each field's buttons there.

use std::collections::BTreeSet;

use kx_module_api::Secret;
use kx_module_api::ports::{EntryKind, FieldButton, FillNote, PageNote};

use super::edit::{Editor, Sub, Taking};
use super::input::shown;
use super::{Desk, Do, Open, Page, Panel};
use crate::book::Notebook;

impl Panel {
    /// The New or Edit form, while it shows without the labels table or the question.
    pub(super) fn form(&mut self) -> Option<&mut Editor> {
        match &mut self.page {
            Page::Edit(edit) if edit.sub == Sub::Form => Some(edit),
            _ => None,
        }
    }

    /// Picks `kind` on New's chips.
    pub(super) fn pick(&mut self, kind: EntryKind) -> Do {
        shown(self.form().is_some_and(|e| e.pick(kind)))
    }

    /// Opens Edit on the shown entry; a saved text no line takes shows the Refused note.
    pub(super) fn edit(&mut self, book: &Notebook) -> Do {
        let Page::Entry(open) = &self.page else {
            return Do::Stay;
        };
        let Some(entry) = book.get(open.id) else {
            return Do::Stay;
        };
        match Editor::of(entry, self.cfg.limits.field_chars) {
            Some(edit) => self.page = Page::Edit(Box::new(edit)),
            None => self.note = Some(PageNote::plain(FillNote::Refused)),
        }
        Do::Show
    }

    /// The name line takes the letters.
    pub(super) fn name(&mut self) -> Do {
        let Some(edit) = self.form().filter(|e| e.kind.is_some()) else {
            return Do::Stay;
        };
        edit.taking = Taking::Name;
        Do::Show
    }

    /// Shows the labels table to add a field, or the Full note at `max_fields`; the shown values hide.
    pub(super) fn table(&mut self) -> Do {
        let most = self.cfg.limits.max_fields;
        let Some(edit) = self.form().filter(|e| e.kind.is_some()) else {
            return Do::Stay;
        };
        if edit.slots.len() < most {
            (edit.sub, edit.shown) = (Sub::Fields(0), BTreeSet::new());
            return Do::Show;
        }
        self.note = Some(PageNote::plain(FillNote::Full));
        Do::Show
    }

    /// Adds a field with the user's own label, from the form or the labels table.
    pub(super) fn own_label(&mut self) -> Do {
        let Page::Edit(edit) = &mut self.page else {
            return Do::Stay;
        };
        if edit.kind.is_none() || edit.sub == Sub::Asking {
            return Do::Stay;
        }
        let tag = edit.own();
        self.add_field(tag)
    }

    /// Saves the form and shows the saved entry's page; a blank form or a refusal keeps the form with its note.
    pub(super) fn save(&mut self, desk: &mut Desk<'_>) -> Do {
        let limits = self.cfg.limits;
        if self.form().is_some_and(|e| e.blank()) {
            self.note = Some(PageNote::plain(FillNote::Refused));
            return Do::Show;
        }
        let Some(edit) = self.form() else {
            return Do::Stay;
        };
        let (Some(draft), id) = (edit.draft(), edit.id) else {
            return Do::Stay;
        };
        let saved = match id {
            None => desk
                .book
                .add(draft, desk.now, desk.random, &limits)
                .map(Some),
            Some(id) => desk
                .book
                .edit(id, draft, desk.now, &limits)
                .map(|kept| kept.then_some(id)),
        };
        match saved {
            Ok(Some(id)) => {
                self.list.id = Some(id);
                self.page = Page::Entry(Open::new(id));
                return Do::Saved;
            }
            Ok(None) => self.page = Page::List,
            Err(error) => self.note = error.note().map(PageNote::plain),
        }
        Do::Show
    }

    /// Asks once whether to delete the edited entry.
    pub(super) fn ask(&mut self) -> Do {
        let Some(edit) = self.form().filter(|e| e.id.is_some()) else {
            return Do::Stay;
        };
        edit.sub = Sub::Asking;
        Do::Show
    }

    /// Answers the question: Yes deletes the entry and shows the list, No goes back to the form.
    pub(super) fn answer(&mut self, yes: bool, book: &mut Notebook) -> Do {
        let Page::Edit(edit) = &mut self.page else {
            return Do::Stay;
        };
        if edit.sub != Sub::Asking {
            return Do::Stay;
        }
        if !yes {
            edit.sub = Sub::Form;
            return Do::Show;
        }
        if let Some(id) = edit.id {
            book.remove(id);
        }
        self.page = Page::List;
        Do::Saved
    }
}

/// A field's `button` on the form, `made` giving a password for Make one; true when it changed the page.
pub(super) fn edit_field(
    edit: &mut Editor,
    i: usize,
    button: FieldButton,
    made: impl FnOnce() -> Option<Secret>,
) -> bool {
    match button {
        FieldButton::Value => edit.focus(i, false),
        FieldButton::Label => edit.focus(i, true),
        FieldButton::Show => edit.show(i) && edit.focus(i, false),
        FieldButton::Hidden => edit.switch(i) && edit.focus(i, false),
        FieldButton::Make => edit.makes(i) && made().is_some_and(|m| edit.made(i, m.expose())),
        FieldButton::Insert | FieldButton::Paste => false,
    }
}
