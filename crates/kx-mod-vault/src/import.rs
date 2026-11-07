//! A `KeePass` import (design "Import"): each entry the worker gives becomes a draft, a Login when it keeps a password
//! and Other when not, every value under the entry rules. A refused value, an entry with no field, a duplicate and what
//! passes a cap are left out and counted, and nothing of the file is logged.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::LazyLock;

use kx_module_api::ports::{
    EntryKind, KeepassEntry, KeepassField, KeepassImport, Label, NoteCounts, Random,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use serde::Deserialize;

use crate::book::{Limits, Notebook};
use crate::entry::{Draft, Entry, Field, Stamp, kept, text};
use crate::find::folded;
use crate::kinds::tables;
use crate::paste::flat;

/// The table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("../import.toml");

/// The table, parsed once; a broken one would name no file and no label, and a test keeps it sound.
static SHEET: LazyLock<Sheet> = LazyLock::new(|| toml::from_str(TABLE).unwrap_or_default());

/// A `KeePass` file's extension, and the labels its standard strings go under.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sheet {
    ext: String,
    user: String,
    password: String,
    website: String,
    text: String,
}

/// The shipped table.
fn sheet() -> &'static Sheet {
    &SHEET
}

/// The extension of a `KeePass` file, which the folder list offers.
#[must_use]
pub fn keepass_ext() -> &'static str {
    &sheet().ext
}

/// What makes two entries one: the kind, and the name and User as search folds them, in wiped buffers.
#[derive(PartialEq, Eq)]
struct Key {
    kind: EntryKind,
    name: Zeroizing<String>,
    user: Zeroizing<String>,
}

impl Key {
    /// The key of an entry of `kind` named `name` with `fields`.
    fn new(kind: EntryKind, name: &str, fields: &[Field]) -> Self {
        let user = fields.iter().find(|f| under(f, &sheet().user));
        Self {
            kind,
            name: folded(name),
            user: user.map(|f| folded(f.value.expose())).unwrap_or_default(),
        }
    }

    /// The key of the saved `entry`.
    fn of_entry(entry: &Entry) -> Self {
        Self::new(entry.kind(), entry.name().expose(), entry.fields())
    }

    /// The key of `draft`.
    fn of_draft(draft: &Draft) -> Self {
        Self::new(draft.kind, draft.name.expose(), &draft.fields)
    }
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.name.as_str().hash(state);
        self.user.as_str().hash(state);
    }
}

/// True when `field` is under table label `id`.
fn under(field: &Field, id: &str) -> bool {
    matches!(&field.label, Label::Table(table) if table == id)
}

/// True when `draft` repeats a key in `seen`, which it joins if new; with no name, nothing says it is the same entry.
fn repeats(seen: &mut HashSet<Key>, draft: &Draft) -> bool {
    let key = Key::of_draft(draft);
    !key.name.is_empty() && !seen.insert(key)
}

/// The fields an entry keeps so far, and how many of its texts the rules refused.
struct Shelf<'a> {
    fields: Vec<Field>,
    refused: u32,
    limits: &'a Limits,
}

impl Shelf<'_> {
    /// Takes `field` under the entry rules: empty it goes, and refused or past `max_fields` it counts.
    fn take(&mut self, field: Field) {
        match kept(field, self.limits.field_chars) {
            Ok(None) => {}
            Ok(Some(field)) if self.fields.len() < self.limits.max_fields => {
                self.fields.push(field);
            }
            Ok(Some(_)) | Err(_) => self.refused = self.refused.saturating_add(1),
        }
    }

    /// Takes `value` under table label `id`, hidden as that label starts.
    fn table(&mut self, id: &str, value: Secret) {
        self.take(Field {
            label: Label::Table(id.to_owned()),
            value,
            hidden: tables().label(id).is_some_and(|l| l.hidden),
        });
    }

    /// Takes a custom string under its own name, hidden, as a file protects one only when asked to. It first
    /// loses the marks a shown text loses, so an Arabic value is never lost.
    fn own(&mut self, field: KeepassField) {
        let KeepassField { label, value } = field;
        let mut value = value.into_inner();
        text::strip(&mut value);
        self.take(Field {
            label: Label::Own(label),
            value: Redacted::new(value),
            hidden: true,
        });
    }

    /// `title` as a name: one the rules refuse counts, and the entry is left unnamed, as its values still matter.
    fn name(&mut self, title: Secret) -> Redacted<String> {
        let mut name = title.into_inner();
        if text::fit(&mut name, false, self.limits.field_chars).is_err() {
            name = Zeroizing::default();
            self.refused = self.refused.saturating_add(1);
        }
        Redacted::new(name.as_str().to_owned())
    }
}

/// `entry` as a draft under the rules, and how many of its texts they refused.
fn mapped(entry: KeepassEntry, limits: &Limits) -> (Draft, u32) {
    let KeepassEntry {
        title,
        user,
        password,
        address,
        notes,
        custom,
    } = entry;
    let sheet = sheet();
    let mut shelf = Shelf {
        fields: Vec::with_capacity(custom.len()),
        refused: 0,
        limits,
    };
    shelf.table(&sheet.user, user);
    shelf.table(&sheet.password, password);
    shelf.table(&sheet.website, address);
    let notes = flat(notes.expose(), false).unwrap_or_default();
    shelf.table(&sheet.text, Redacted::new(notes));
    for field in custom {
        shelf.own(field);
    }
    let name = shelf.name(title);
    let login = shelf.fields.iter().any(|f| under(f, &sheet.password));
    let kind = if login {
        EntryKind::Login
    } else {
        EntryKind::Other
    };
    let fields = shelf.fields;
    (Draft { name, kind, fields }, shelf.refused)
}

/// Brings `import` into `book`, changed at `now`, with new ids from `random`; gives how many entries came in and how
/// many entries and fields were left out, the worker's own drops among them.
pub fn bring_in(
    book: &mut Notebook,
    import: KeepassImport,
    now: Stamp,
    random: &dyn Random,
    limits: &Limits,
) -> NoteCounts {
    let KeepassImport { entries, dropped } = import;
    let mut skipped = dropped.entries.saturating_add(dropped.fields);
    let mut seen: HashSet<Key> = book.entries().iter().map(Key::of_entry).collect();
    let mut drafts = Vec::with_capacity(entries.len());
    for entry in entries {
        let (draft, refused) = mapped(entry, limits);
        skipped = skipped.saturating_add(refused);
        if draft.fields.is_empty() || repeats(&mut seen, &draft) {
            skipped = skipped.saturating_add(1);
        } else {
            drafts.push(draft);
        }
    }
    let wanted = u32::try_from(drafts.len()).unwrap_or(u32::MAX);
    let added = book.add_all(drafts, now, random, limits);
    NoteCounts {
        added,
        skipped: skipped.saturating_add(wanted.saturating_sub(added)),
    }
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
