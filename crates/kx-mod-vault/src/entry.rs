//! Entries (design "The notebook"): what New and Edit hand over (`Draft`), and the saved `Entry`, which obeys every
//! rule. Times are wall times (`Stamp`), as Bring back compares two computers' copies and the use order outlives a
//! restart; the monotonic `Clock` starts again with the PC.

mod line;
pub mod text;

pub use line::Line;

use kx_module_api::ports::{EntryKind, FillNote, Label, LocalTime, RandomError};
use kx_module_api::{Redacted, Secret, Zeroizing};
use thiserror::Error;

use crate::book::Limits;
use crate::kinds::tables;

/// A wall time: milliseconds since the Unix epoch, in UTC, as the `Calendar` port tells it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Stamp(i64);

impl Stamp {
    /// The time `unix_ms` milliseconds after the Unix epoch.
    #[must_use]
    pub const fn from_unix_ms(unix_ms: i64) -> Self {
        Self(unix_ms)
    }

    /// The time of the wall time `now`.
    #[must_use]
    pub const fn of(now: &LocalTime) -> Self {
        Self(now.unix_ms)
    }

    /// One millisecond after it.
    #[must_use]
    pub const fn after(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Milliseconds since the Unix epoch.
    #[must_use]
    pub const fn unix_ms(self) -> i64 {
        self.0
    }
}

/// Why an entry was not saved.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Error)]
pub enum EntryError {
    /// A value, the name or an own label is too long, breaks a line or holds a hidden mark, an own label is blank, or a
    /// table label is unknown.
    #[error("a text is too long, breaks a line or holds a hidden mark")]
    Refused,
    /// The notebook or the entry is full.
    #[error("the notebook or the entry is full")]
    Full,
    /// The system gave no random bytes for a new id.
    #[error("the system gave no random bytes")]
    NoRandom,
}

impl EntryError {
    /// The panel's note for it; a missing random source has none, as only a broken system lacks one.
    #[must_use]
    pub const fn note(self) -> Option<FillNote> {
        match self {
            Self::Refused => Some(FillNote::Refused),
            Self::Full => Some(FillNote::Full),
            Self::NoRandom => None,
        }
    }
}

impl From<RandomError> for EntryError {
    fn from(_: RandomError) -> Self {
        Self::NoRandom
    }
}

/// A field: its label, its value, wiped when dropped, and whether it is hidden.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// Its label.
    pub label: Label,
    /// Its value.
    pub value: Secret,
    /// It is hidden, so pages show the mask and search skips it.
    pub hidden: bool,
}

/// What New or Edit hands over, before the rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Draft {
    /// The entry's name.
    pub name: Redacted<String>,
    /// Its kind.
    pub kind: EntryKind,
    /// Its fields, empty ones too.
    pub fields: Vec<Field>,
}

impl Draft {
    /// A new entry of `kind`, unnamed, with its kind's starter fields, empty, hidden as their labels start.
    #[must_use]
    pub fn new(kind: EntryKind) -> Self {
        let ids = tables()
            .kind(kind)
            .map(|r| r.fields.as_slice())
            .unwrap_or_default();
        let fields = ids.iter().map(|id| Field {
            label: Label::Table(id.clone()),
            value: Redacted::new(Zeroizing::default()),
            hidden: tables().label(id).is_some_and(|l| l.hidden),
        });
        Self {
            name: Redacted::default(),
            kind,
            fields: fields.collect(),
        }
    }
}

/// A saved entry: a stable id, its name, kind and kept fields, when it last changed and when it was last filled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    id: u64,
    name: Redacted<String>,
    kind: EntryKind,
    fields: Vec<Field>,
    changed: Stamp,
    used: Option<Stamp>,
}

impl Entry {
    /// Entry `id` from `draft`, changed at `now`, under the rules: empty fields go, each text fits, at most
    /// `max_fields` fields stay.
    ///
    /// # Errors
    /// `Refused` for a text that does not fit, a blank own label or an unknown table label, then `Full` past
    /// `max_fields`.
    pub fn new(id: u64, draft: Draft, now: Stamp, limits: &Limits) -> Result<Self, EntryError> {
        let max = limits.field_chars;
        let mut name = draft.name.into_inner();
        text::fit(&mut name, false, max)?;
        let mut fields = Vec::with_capacity(draft.fields.len());
        for field in draft.fields {
            if let Some(field) = kept(field, max)? {
                fields.push(field);
            }
        }
        if fields.len() > limits.max_fields {
            return Err(EntryError::Full);
        }
        let name = Redacted::new(name);
        Ok(Self {
            id,
            name,
            kind: draft.kind,
            fields,
            changed: now,
            used: None,
        })
    }

    /// An entry the store read back, whose texts it has checked.
    pub(crate) fn restored(id: u64, draft: Draft, changed: Stamp, used: Option<Stamp>) -> Self {
        let Draft { name, kind, fields } = draft;
        Self {
            id,
            name,
            kind,
            fields,
            changed,
            used,
        }
    }

    /// Its stable id.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Its name.
    #[must_use]
    pub const fn name(&self) -> &Redacted<String> {
        &self.name
    }

    /// Its kind.
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Its fields, none empty.
    #[must_use]
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// When it last changed.
    #[must_use]
    pub const fn changed(&self) -> Stamp {
        self.changed
    }

    /// When it was last filled, if ever.
    #[must_use]
    pub const fn used(&self) -> Option<Stamp> {
        self.used
    }

    /// Its draft, for Edit.
    #[must_use]
    pub fn draft(&self) -> Draft {
        Draft {
            name: self.name.clone(),
            kind: self.kind,
            fields: self.fields.clone(),
        }
    }

    /// Notes that it was filled at `now`.
    pub(crate) fn use_at(&mut self, now: Stamp) {
        self.used = Some(now);
    }
}

/// `field` under the rules: `None` once its value is empty, else its label and value fitted.
pub(crate) fn kept(field: Field, max: usize) -> Result<Option<Field>, EntryError> {
    let Field {
        label,
        value,
        hidden,
    } = field;
    let mut value = value.into_inner();
    text::fit(&mut value, hidden, max)?;
    if value.is_empty() {
        return Ok(None);
    }
    let label = match label {
        Label::Table(id) if tables().label(&id).is_some() => Label::Table(id),
        Label::Table(_) => return Err(EntryError::Refused),
        Label::Own(own) => {
            let mut own = own.into_inner();
            text::fit(&mut own, false, max)?;
            if text::blank(&own) {
                return Err(EntryError::Refused);
            }
            Label::Own(Redacted::new(own))
        }
    };
    Ok(Some(Field {
        label,
        value: Redacted::new(value),
        hidden,
    }))
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
