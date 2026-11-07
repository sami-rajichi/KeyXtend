//! The notebook's pages as QML reads them: the list, By label and its values, one entry, New and Edit, and the labels
//! table to add a field. Each row's buttons are worded for what they do on that page.

use super::fields::{Edit, Field, edit, field, shown};
use super::{Button, Drawn, backup, button, label, text, word};
use crate::I18n;
use crate::i18n::keys::{
    FILL_ADD, FILL_JOIN, FILL_KIND, FILL_MASK, FILL_NONE, FILL_VALUES, entry_kind_id, row_button_id,
};
use kx_module_api::ports::{
    EntryKind, EntryRow, FillPage, Hint, Label, LabelCount, LabelValue, RowButton, Sheet,
};
use serde::Serialize;

/// The page shown, tagged by what it is.
#[derive(Serialize)]
#[serde(tag = "is", rename_all = "kebab-case")]
pub(super) enum Shown {
    List {
        rows: Vec<Row>,
        none: Option<String>,
    },
    Labels {
        rows: Vec<Counted>,
        none: Option<String>,
    },
    Values {
        label: Drawn,
        rows: Vec<Valued>,
    },
    Entry {
        kind: &'static str,
        name: Drawn,
        fields: Vec<Field>,
    },
    Edit(Edit),
    AddField {
        rows: Vec<Counted>,
    },
    Backup(backup::Backup),
    Places(backup::Spots),
    Folders(backup::Folders),
    Password(backup::Password),
    Found {
        rows: Vec<backup::Found>,
    },
    Foreign(backup::Found),
}

/// An entry's row; its id is a string, as JavaScript numbers lose a large one.
#[derive(Serialize)]
pub(super) struct Row {
    id: String,
    kind: &'static str,
    name: Drawn,
    hint: Drawn,
    buttons: Vec<Button>,
}

/// A label with how many values carry it, or none on the labels table.
#[derive(Serialize)]
pub(super) struct Counted {
    label: Drawn,
    values: Option<String>,
    buttons: Vec<Button>,
}

/// A value under a label, with its entry's name.
#[derive(Serialize)]
pub(super) struct Valued {
    name: Drawn,
    value: Option<Drawn>,
    buttons: Vec<Button>,
}

/// `page`'s sheet with every text in `t`.
pub(super) fn of(page: &FillPage, t: &I18n) -> Shown {
    let none = || (!page.empty).then(|| t.text(FILL_NONE));
    match &page.sheet {
        Sheet::List(rows) => Shown::List {
            none: rows.is_empty().then(none).flatten(),
            rows: rows.iter().map(|r| row(r, t)).collect(),
        },
        Sheet::Labels(rows) => Shown::Labels {
            none: rows.is_empty().then(none).flatten(),
            rows: rows.iter().map(|r| by_label(r, t)).collect(),
        },
        Sheet::Values { label: l, rows } => Shown::Values {
            label: label(l, t),
            rows: rows.iter().map(|r| valued(r, t)).collect(),
        },
        Sheet::Entry(e) => Shown::Entry {
            kind: entry_kind_id(e.kind),
            name: named(e.name.expose(), e.kind, t),
            fields: e.fields.iter().map(|f| field(f, None, t)).collect(),
        },
        Sheet::Edit(e) => Shown::Edit(edit(e, t)),
        Sheet::AddField(labels) => Shown::AddField {
            rows: labels.iter().map(|l| counted(l, None, t)).collect(),
        },
        Sheet::Backup(b) => Shown::Backup(backup::backup(b, t)),
        Sheet::Places(spots) => Shown::Places(backup::places(spots, t)),
        Sheet::Folders(f) => Shown::Folders(backup::folders(f, t)),
        Sheet::Password(p) => Shown::Password(backup::password(p, t)),
        Sheet::Found(rows) => Shown::Found {
            rows: rows.iter().map(|r| backup::found(r, t)).collect(),
        },
        Sheet::Foreign(r) => Shown::Foreign(backup::foreign(r, t)),
        Sheet::ForeignUndated(place) => Shown::Foreign(backup::foreign_undated(place, t)),
    }
}

/// By label's row `r`: the label, how many values carry it, and Open.
fn by_label(r: &LabelCount, t: &I18n) -> Counted {
    let n = usize::try_from(r.count).unwrap_or(usize::MAX);
    counted(&r.label, Some(t.counted(FILL_VALUES, n)), t)
}

/// A value under the label, with its entry's name and Insert.
fn valued(r: &LabelValue, t: &I18n) -> Valued {
    Valued {
        name: text(r.name.expose()),
        value: shown(&r.value),
        buttons: vec![row_button(t, RowButton::Insert)],
    }
}

/// Entry row `r`, its hint glimpsing values with the mask or counting fields after its kind's name.
fn row(r: &EntryRow, t: &I18n) -> Row {
    let hint = match &r.hint {
        Hint::Values(values) => {
            let mask = t.text(FILL_MASK);
            let shown = values
                .iter()
                .map(|v| v.text().map_or(mask.clone(), |s| s.expose().to_string()));
            text(&shown.collect::<Vec<_>>().join(&t.text(FILL_JOIN)))
        }
        Hint::Fields(n) => text(&t.fill_fields(&kind_name(r.kind, t), usize::from(*n))),
    };
    Row {
        id: r.id.to_string(),
        kind: entry_kind_id(r.kind),
        name: named(r.name.expose(), r.kind, t),
        hint,
        buttons: r.buttons.iter().map(|&b| row_button(t, b)).collect(),
    }
}

/// A label's row: By label's with its count of `values` and Open, or the labels table's with Add.
fn counted(l: &Label, values: Option<String>, t: &I18n) -> Counted {
    let open = row_button_id(RowButton::Open);
    let add = values.is_none();
    Counted {
        label: label(l, t),
        values,
        buttons: vec![if add {
            button(t, open, FILL_ADD)
        } else {
            row_button(t, RowButton::Open)
        }],
    }
}

/// An entry's name `raw` to draw; one empty, or hidden marks alone, shows its kind's name.
fn named(raw: &str, kind: EntryKind, t: &I18n) -> Drawn {
    let own = text(raw);
    if own.text.is_empty() {
        text(&kind_name(kind, t))
    } else {
        own
    }
}

/// Kind `kind`'s name.
fn kind_name(kind: EntryKind, t: &I18n) -> String {
    t.text(&format!("{FILL_KIND}{}", entry_kind_id(kind)))
}

/// Row button `b`, labelled as itself.
pub(super) fn row_button(t: &I18n, b: RowButton) -> Button {
    Button {
        id: row_button_id(b),
        label: word(t, row_button_id(b)),
    }
}
