//! A field's line and the New or Edit page as QML reads them: each line's label and value, the mask while a hidden
//! value is not shown and on the form how many dots draw it, its buttons for the page, and the line that takes the
//! letters.

use super::{Button, Drawn, button, label, text, word};
use crate::I18n;
use crate::i18n::keys::{
    FILL_CONFIRM, FILL_EDIT_TITLE, FILL_HIDE, FILL_KIND, FILL_KINDS, FILL_NEW_TITLE, FILL_WORD,
    entry_kind_id, field_button_id,
};
use kx_module_api::ports::{
    EditSheet, EntryKind, FieldButton, FieldLine, FieldValue, Label, Typing,
};
use serde::Serialize;

/// A field's line: its label, and its value or `null` for the mask.
#[derive(Serialize)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "Each flag is one fact QML draws on its own: an own label, hidden, shown, switching and Make one."
)]
pub(super) struct Field {
    label: Drawn,
    own: bool,
    value: Option<Drawn>,
    /// The dots a masked value draws on the form, one per character; on a read page, `null`.
    count: Option<usize>,
    hidden: bool,
    shown: bool,
    switches: bool,
    make: bool,
    /// The line that takes the letters, `value` or `label`, if one does.
    typing: Option<&'static str>,
    buttons: Vec<Button>,
}

/// The New or Edit page.
#[derive(Serialize)]
pub(super) struct Edit {
    title: String,
    new: bool,
    kind: Option<&'static str>,
    /// What New asks over its kind chips, until a kind is picked.
    ask: Option<String>,
    kinds: Vec<Button>,
    /// The name line takes the letters.
    typing: bool,
    /// The caret of the line that takes the letters: UTF-16 units of its text, or dots while masked.
    caret: u32,
    name: Drawn,
    fields: Vec<Field>,
    confirm: Option<String>,
}

/// The New or Edit page `e`; New shows the kind chips until a kind is picked.
pub(super) fn edit(e: &EditSheet, t: &I18n) -> Edit {
    let chip = |k: EntryKind| {
        button(
            t,
            entry_kind_id(k),
            &format!("{FILL_KIND}{}", entry_kind_id(k)),
        )
    };
    let kinds = if e.kind.is_some() {
        Vec::new()
    } else {
        EntryKind::ALL.into_iter().map(chip).collect()
    };
    Edit {
        title: t.text(if e.new {
            FILL_NEW_TITLE
        } else {
            FILL_EDIT_TITLE
        }),
        new: e.new,
        kind: e.kind.map(entry_kind_id),
        ask: e.kind.is_none().then(|| t.text(FILL_KINDS)),
        kinds,
        typing: e.typing == Some(Typing::Name),
        caret: *e.caret.expose(),
        name: text(e.name.expose()),
        fields: (0..)
            .zip(&e.fields)
            .map(|(at, f)| field(f, typing(e.typing, at), t))
            .collect(),
        confirm: e.confirm.then(|| t.text(FILL_CONFIRM)),
    }
}

/// The line of the field at `at` that takes the letters, by `typing`.
fn typing(typing: Option<Typing>, at: u8) -> Option<&'static str> {
    match typing? {
        Typing::Value(n) if n == at => Some(field_button_id(FieldButton::Value)),
        Typing::Label(n) if n == at => Some(field_button_id(FieldButton::Label)),
        _ => None,
    }
}

/// Field line `f`, its line `typing` the letters if any. A read page offers Insert, then Show or Hide for a hidden
/// value; New and Edit offer Show or Hide, the hidden switch, Paste and Make one as the field allows.
pub(super) fn field(f: &FieldLine, typing: Option<&'static str>, t: &I18n) -> Field {
    let v = &f.value;
    let show = v.hidden().then(|| {
        let key = if v.shown() {
            FILL_HIDE.to_owned()
        } else {
            format!("{FILL_WORD}{}", field_button_id(FieldButton::Show))
        };
        button(t, field_button_id(FieldButton::Show), &key)
    });
    let mut buttons: Vec<Button> = Vec::new();
    if !v.switches() {
        buttons.push(field_button(t, FieldButton::Insert));
    }
    buttons.extend(show);
    if v.switches() {
        buttons.push(field_button(t, FieldButton::Hidden));
        buttons.push(field_button(t, FieldButton::Paste));
        buttons.extend(f.make.then(|| field_button(t, FieldButton::Make)));
    }
    Field {
        label: label(&f.label, t),
        own: matches!(f.label, Label::Own(_)),
        value: shown(v),
        count: v.count(),
        hidden: v.hidden(),
        shown: v.shown(),
        switches: v.switches(),
        make: f.make,
        typing,
        buttons,
    }
}

/// Value `v` to draw, or `None` for the mask.
pub(super) fn shown(v: &FieldValue) -> Option<Drawn> {
    v.text().map(|s| text(s.expose()))
}

/// Field button `b`, labelled as itself.
fn field_button(t: &I18n, b: FieldButton) -> Button {
    Button {
        id: field_button_id(b),
        label: word(t, field_button_id(b)),
    }
}
