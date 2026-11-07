//! Clicks on the panel: a row's or a field's button by its place on the view the user saw, a filter, a kind, a page
//! turn, a panel button or an answer. A click from an older view is dropped unless `kept` says it acts the same on any
//! view, and only clicks change the notebook.

use kx_module_api::ports::{FieldButton, FillAct, FillButton, FillClick, RowButton};

use super::edit::{Sub, Tag};
use super::form::edit_field;
use super::input::{one, shown};
use super::rows::buttons;
use super::{Cursor, Desk, Do, Open, Page, Panel, Target};
use crate::book::Notebook;
use crate::make::make;

impl Panel {
    /// Takes a click on the view of round `act.round`; the ui's rows count whatever the view.
    pub fn act(&mut self, act: FillAct, desk: &mut Desk<'_>) -> Do {
        if let FillClick::Rows(rows) = act.click {
            return self.set_rows(rows);
        }
        if !self.is_open() || (act.round != self.round && !kept(act.click)) {
            return Do::Stay;
        }
        if self.sync(desk.book) {
            return Do::Show;
        }
        if self.on_backup() {
            return self.backup_click(act.click);
        }
        self.noted(|p| p.click(act.click, desk))
    }

    /// Does what `click` asks on the shown page; Delete entry's question takes only its answer and Close.
    fn click(&mut self, click: FillClick, desk: &mut Desk<'_>) -> Do {
        let lists = matches!(self.page, Page::List | Page::Values { .. });
        let asking = matches!(&self.page, Page::Edit(e) if e.sub == Sub::Asking);
        let answers = matches!(
            click,
            FillClick::Answer(_) | FillClick::Button(FillButton::Close)
        );
        if asking && !answers {
            return Do::Stay;
        }
        match click {
            FillClick::Row { row, button } => self.row(row, button, desk.book),
            FillClick::Field { field, button } => self.field(field, button, desk),
            FillClick::Filter(filter) if lists => self.set_filter(filter),
            FillClick::Kind(kind) => self.pick(kind),
            FillClick::Page { back } => self.turn(back, desk.book),
            FillClick::Button(button) => self.button(button, desk),
            FillClick::Answer(yes) => self.answer(yes, desk.book),
            FillClick::Caret { at } => self.place_caret(at.into_inner()),
            FillClick::Filter(_) | FillClick::Checks(_) | FillClick::Rows(_) => Do::Stay,
        }
    }

    /// Pages the lists by `rows`, at least one, kept while closed.
    fn set_rows(&mut self, rows: u8) -> Do {
        let rows = usize::from(rows).max(1);
        if rows == self.rows() {
            return Do::Stay;
        }
        self.cfg.rows = rows;
        shown(self.is_open())
    }

    /// Row `row`'s `button` on the view the user saw: it highlights that row, then acts when the row offers it.
    fn row(&mut self, row: u8, button: RowButton, book: &Notebook) -> Do {
        let place = usize::from(row);
        let Some(target) = self.seen.rows.get(place).cloned() else {
            return Do::Stay;
        };
        let at = self.seen.start + place;
        match (&self.page, target, button) {
            (Page::List, Target::Entry(id), _) => {
                let Some(entry) = book.get(id).filter(|e| buttons(e).contains(&button)) else {
                    return Do::Stay;
                };
                self.list = Cursor { at, id: Some(id) };
                self.press(entry, button)
            }
            (Page::List, Target::Label(label), RowButton::Open) => {
                self.list.at = at;
                self.open_label(label)
            }
            (Page::Values { .. }, Target::Value(id, i), RowButton::Insert) => {
                if book.get(id).is_none_or(|e| i >= e.fields().len()) {
                    return Do::Stay;
                }
                self.put(at, &[]);
                one(id, i)
            }
            (Page::Edit(e), Target::Table(id), RowButton::Open)
                if matches!(e.sub, Sub::Fields(_)) =>
            {
                self.add_field(Tag::Table(id))
            }
            _ => Do::Stay,
        }
    }

    /// Field `field`'s `button` on the view the user saw.
    fn field(&mut self, field: u8, button: FieldButton, desk: &mut Desk<'_>) -> Do {
        let Some(Target::Field(i)) = self.seen.rows.get(usize::from(field)).cloned() else {
            return Do::Stay;
        };
        let length = self.cfg.make_chars;
        match &mut self.page {
            Page::Entry(open) => read_field(open, i, button, desk.book),
            Page::Edit(edit) if edit.sub == Sub::Form => {
                if button == FieldButton::Paste {
                    let asked = edit.focus(i, false);
                    edit.pasting = asked.then_some(i);
                    return if asked { Do::Paste } else { Do::Stay };
                }
                let made = || make(length, desk.random).ok();
                shown(edit_field(edit, i, button, made))
            }
            _ => Do::Stay,
        }
    }

    /// A panel button.
    fn button(&mut self, button: FillButton, desk: &mut Desk<'_>) -> Do {
        match button {
            FillButton::Close => self.shut(),
            FillButton::New => self.start_new(),
            FillButton::Back => self.back(),
            FillButton::Edit => self.edit(desk.book),
            FillButton::Name => self.name(),
            FillButton::AddField => self.table(),
            FillButton::OwnLabel => self.own_label(),
            FillButton::Save => self.save(desk),
            FillButton::Cancel if self.form().is_some() => self.cancel(),
            FillButton::Delete => self.ask(),
            FillButton::Backup => self.open_backup(),
            FillButton::Import => self.open_import(desk.book),
            // Cancel past the form, and the backup and import pages' buttons, which those pages take.
            FillButton::Cancel
            | FillButton::BackUpNow
            | FillButton::AddPlace
            | FillButton::ChangePassword
            | FillButton::BringBack
            | FillButton::OtherFolder
            | FillButton::Here
            | FillButton::Done => Do::Stay,
        }
    }
}

/// A field's `button` on an entry page: Insert asks its fill, Show shows or hides a hidden value; each highlights it.
fn read_field(open: &mut Open, i: usize, button: FieldButton, book: &Notebook) -> Do {
    let Some(field) = book.get(open.id).and_then(|e| e.fields().get(i)) else {
        return Do::Stay;
    };
    match button {
        FieldButton::Insert => {
            open.at = i;
            one(open.id, i)
        }
        FieldButton::Show if field.hidden => {
            open.at = i;
            if !open.shown.remove(&i) {
                open.shown.insert(i);
            }
            Do::Show
        }
        _ => Do::Stay,
    }
}

/// True when a click from an older view is still safe to take: it names no place on the page and neither types, saves,
/// cancels, deletes, answers nor adds a field, so it acts the same on any view and a shaky second tap adds nothing.
/// A caret click comes right after its line's own click, whose new view the ui has not drawn yet.
const fn kept(click: FillClick) -> bool {
    match click {
        FillClick::Filter(_)
        | FillClick::Kind(_)
        | FillClick::Page { .. }
        | FillClick::Rows(_)
        | FillClick::Caret { .. } => true,
        FillClick::Button(button) => matches!(
            button,
            FillButton::Close
                | FillButton::New
                | FillButton::Back
                | FillButton::Name
                | FillButton::AddField
        ),
        FillClick::Row { .. }
        | FillClick::Field { .. }
        | FillClick::Answer(_)
        | FillClick::Checks(_) => false,
    }
}
