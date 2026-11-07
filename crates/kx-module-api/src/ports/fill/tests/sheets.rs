//! One sample of every sheet, each name, value, search and place one of the secrets.

use super::*;
use crate::ports::{Date, Pages, PlaceKind, When};

/// A caret that appears nowhere else.
pub(super) const CARET: u32 = 73_519;

/// A hidden field that shows secret 0 and offers Make one, and a plain email of secret 1, each made by `value`.
pub(super) fn fields(value: fn(Secret, bool, bool) -> FieldValue) -> Vec<FieldLine> {
    let shown = FieldLine {
        label: Label::Own(text(1)),
        value: value(secret(0), true, true),
        make: true,
    };
    let email = FieldLine {
        label: Label::Table("email".to_owned()),
        value: value(secret(1), false, false),
        make: false,
    };
    vec![shown, email]
}

/// The secret place, a cloud folder.
fn spot() -> Spot {
    Spot {
        kind: PlaceKind::Cloud,
        name: text(2),
    }
}

/// A box in the secret place, written on 1 Nov 2026 at 10:42.
fn found() -> FoundRow {
    let when = When {
        days_ago: 0,
        date: Date::new(2026, 11, 1).unwrap(),
        hour: 10,
        minute: 42,
    };
    FoundRow {
        place: spot(),
        when,
    }
}

/// A login row of the two secret values.
fn login() -> EntryRow {
    EntryRow {
        id: 7,
        kind: EntryKind::Login,
        name: text(1),
        hint: Hint::Values(
            fields(FieldValue::new)
                .into_iter()
                .map(|f| f.value)
                .collect(),
        ),
        buttons: vec![RowButton::User, RowButton::Password, RowButton::Both],
    }
}

/// An Edit page of the login, typing into its first value, with the Delete question open.
fn edit() -> EditSheet {
    EditSheet {
        new: false,
        kind: Some(EntryKind::Login),
        name: text(1),
        fields: fields(FieldValue::edit),
        typing: Some(Typing::Value(0)),
        caret: Redacted::new(CARET),
        confirm: true,
    }
}

/// The backup page, with the secret place updated.
fn backup() -> BackupSheet {
    let state = BoxState::Updated(found().when);
    BackupSheet {
        places: vec![PlaceRow {
            place: spot(),
            state,
        }],
        checks: true,
        ask: Some(BackupAsk::Remove(0)),
    }
}

/// One sheet of each kind, in declaration order.
pub(super) fn sheets() -> Vec<Sheet> {
    let values = vec![LabelValue {
        name: text(1),
        value: FieldValue::new(secret(0), false, false),
    }];
    let entry = EntrySheet {
        kind: EntryKind::Wifi,
        name: text(1),
        fields: fields(FieldValue::new),
    };
    let folders = FolderSheet {
        pick: Pick::Keepass,
        folder: Some(text(2)),
        rows: vec![spot()],
    };
    let password = PasswordSheet {
        ask: PasswordAsk::Open { own: true },
        text: secret(0),
        caret: Redacted::new(CARET),
    };
    vec![
        Sheet::List(vec![login()]),
        Sheet::Labels(vec![LabelCount {
            label: Label::Own(text(0)),
            count: 2,
        }]),
        Sheet::Values {
            label: Label::Own(text(1)),
            rows: values,
        },
        Sheet::Entry(entry),
        Sheet::Edit(edit()),
        Sheet::AddField(vec![Label::Own(text(0))]),
        Sheet::Backup(backup()),
        Sheet::Places(vec![spot()]),
        Sheet::Folders(folders),
        Sheet::Password(password),
        Sheet::Found(vec![found()]),
        Sheet::Foreign(found()),
        Sheet::ForeignUndated(spot()),
    ]
}

/// The place of `sheet` in `Sheet`'s declaration; a new sheet fails to build here until it is sampled above.
pub(super) const fn place(sheet: &Sheet) -> usize {
    match sheet {
        Sheet::List(_) => 0,
        Sheet::Labels(_) => 1,
        Sheet::Values { .. } => 2,
        Sheet::Entry(_) => 3,
        Sheet::Edit(_) => 4,
        Sheet::AddField(_) => 5,
        Sheet::Backup(_) => 6,
        Sheet::Places(_) => 7,
        Sheet::Folders(_) => 8,
        Sheet::Password(_) => 9,
        Sheet::Found(_) => 10,
        Sheet::Foreign(_) => 11,
        Sheet::ForeignUndated(_) => 12,
    }
}

/// A finished import's note, with its counts.
fn imported() -> PageNote {
    let counts = NoteCounts {
        added: 3,
        skipped: 1,
    };
    PageNote {
        note: FillNote::Imported,
        counts,
    }
}

/// A page showing `sheet`, searched for secret 1, with a note and waiting for the secret place.
pub(super) fn page(sheet: Sheet) -> FillPage {
    FillPage {
        round: 3,
        sheet,
        filter: FillFilter::All,
        search: text(1),
        caret: Redacted::new(CARET),
        pages: Pages { at: 0, count: 1 },
        highlight: Some(0),
        note: Some(imported()),
        waiting: Some(spot()),
        empty: false,
    }
}
