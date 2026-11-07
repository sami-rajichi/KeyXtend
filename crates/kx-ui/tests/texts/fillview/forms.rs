//! Quick-fill's entry pages and its New and Edit pages for the fixture: a login's page masked, shown and with an empty
//! hidden value, an Arabic person's page, New's kinds and form, Edit with its question, and the labels table to add a
//! field.

use super::{ARABIC, page, secret, shown, table};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    EditSheet, EntryKind, EntrySheet, FieldLine, FieldValue, FillPage, Label, Sheet, Typing,
};

/// A read page's line labelled `label` holding `text`, hidden when `hidden` and shown when `open`.
fn line(label: Label, text: &str, hidden: bool, open: bool) -> FieldLine {
    FieldLine {
        label,
        value: FieldValue::new(secret(text), hidden, open),
        make: false,
    }
}

/// A login's page with its password masked, shown and empty; and an Arabic person's page with an own label.
pub(super) fn entries() -> Vec<(&'static str, FillPage)> {
    let login = |open| EntrySheet {
        kind: EntryKind::Login,
        name: shown("GitHub"),
        fields: vec![
            line(table("user"), "alex", false, false),
            line(table("password"), "s3cret-Pass!", true, open),
            line(table("website"), "github.com", false, false),
        ],
    };
    let arabic = EntrySheet {
        kind: EntryKind::Person,
        name: shown(ARABIC[0]),
        fields: vec![
            line(table("full-name"), ARABIC[0], false, false),
            line(table("email"), ARABIC[1], false, false),
            line(Label::Own(shown(ARABIC[2])), ARABIC[3], false, false),
        ],
    };
    let blank = EntrySheet {
        kind: EntryKind::Login,
        name: shown("GitHub"),
        fields: vec![
            line(table("user"), "alex", false, false),
            line(table("password"), "", true, false),
        ],
    };
    vec![
        ("entry", page(Sheet::Entry(login(false)), 1)),
        ("shown", page(Sheet::Entry(login(true)), 1)),
        ("blank", page(Sheet::Entry(blank), 1)),
        ("arabic", page(Sheet::Entry(arabic), 1)),
    ]
}

/// A New or Edit line labelled `label` holding `text`, hidden when `hidden`, shown when `open`, with Make one on `make`.
fn edit_line(label: Label, text: &str, hidden: bool, open: bool, make: bool) -> FieldLine {
    FieldLine {
        label,
        value: FieldValue::edit(secret(text), hidden, open),
        make,
    }
}

/// New's login form: its starting lines, empty.
fn starters() -> Vec<FieldLine> {
    vec![
        edit_line(table("user"), "", false, false, false),
        edit_line(table("password"), "", true, false, true),
        edit_line(table("website"), "", false, false, false),
    ]
}

/// A login on Edit: its password shown, and a hidden own label that tells its count.
fn fields() -> Vec<FieldLine> {
    vec![
        edit_line(table("user"), "alex", false, false, false),
        edit_line(table("password"), "s3cret-Pass!", true, true, true),
        edit_line(Label::Own(shown("Recovery")), "XK2-99", true, false, false),
        edit_line(table("website"), "github.com", false, false, false),
    ]
}

/// New with its kinds, New's form with the name taking the letters, Edit with a value taking them, its question, and
/// the labels table to add a field.
pub(super) fn edits() -> Vec<(&'static str, FillPage)> {
    let sheet = |new, kind, name: &str, fields, typing| EditSheet {
        new,
        kind,
        name: shown(name),
        fields,
        typing,
        caret: Redacted::new(0),
        confirm: false,
    };
    let login = Some(EntryKind::Login);
    let editing = sheet(false, login, "GitHub", fields(), Some(Typing::Value(1)));
    let mut edit = page(Sheet::Edit(editing.clone()), 1);
    edit.highlight = Some(1);
    let asking = EditSheet {
        confirm: true,
        ..editing
    };
    let add = (["full-name", "email", "phone", "address", "city"]).map(table);
    vec![
        (
            "new",
            page(Sheet::Edit(sheet(true, None, "", Vec::new(), None)), 1),
        ),
        (
            "form",
            page(
                Sheet::Edit(sheet(true, login, "", starters(), Some(Typing::Name))),
                1,
            ),
        ),
        ("edit", edit),
        ("confirm", page(Sheet::Edit(asking), 1)),
        ("add", page(Sheet::AddField(add.to_vec()), 4)),
    ]
}
