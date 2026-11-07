//! What the unit tests share: the default limits, and drafts and fields built from plain text.

use kx_module_api::ports::{EntryKind, Label, MAX_BLOB_BYTES};
use kx_module_api::{Redacted, Secret, Zeroizing};

use crate::book::Limits;
use crate::entry::{Draft, Field};

/// The limits of the default settings.
pub const LIMITS: Limits = Limits {
    field_chars: 1000,
    max_entries: 2000,
    max_fields: 30,
    max_bytes: MAX_BLOB_BYTES,
};

/// `text` as a secret.
pub fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// A field under table label `id` holding `text`.
pub fn field(id: &str, text: &str, hidden: bool) -> Field {
    Field {
        label: Label::Table(id.to_owned()),
        value: secret(text),
        hidden,
    }
}

/// A field under the user's own `label` holding `text`.
pub fn own(label: &str, text: &str, hidden: bool) -> Field {
    Field {
        label: Label::Own(Redacted::new(label.to_owned())),
        value: secret(text),
        hidden,
    }
}

/// A draft of `kind` named `name` with `fields`.
pub fn draft(name: &str, kind: EntryKind, fields: Vec<Field>) -> Draft {
    Draft {
        name: Redacted::new(name.to_owned()),
        kind,
        fields,
    }
}
