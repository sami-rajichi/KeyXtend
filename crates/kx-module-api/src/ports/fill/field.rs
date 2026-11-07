//! A field as a page shows it: its label, and its value or the mask. A hidden value with text is carried only while it
//! is shown, and only New and Edit let its hidden flag switch and tell how many characters the mask covers.

use crate::{Redacted, Secret};

/// A field's label: one from the module's labels table, or the user's own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label {
    /// A table label by its id, such as `email`; the ui shows its text.
    Table(String),
    /// The user's own label.
    Own(Redacted<String>),
}

/// A field's value as a page shows it: a hidden one with text is carried only while shown, so the page holds no copy
/// otherwise; an empty one has nothing to mask.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldValue {
    /// The text, or `None` for the mask.
    text: Option<Secret>,
    /// The characters under the mask, on New and Edit only; redacted, as it is a secret's length.
    count: Option<Redacted<usize>>,
    /// The field is hidden.
    hidden: bool,
    /// The field is hidden and shown now.
    shown: bool,
    /// Its hidden flag can switch, on the New or Edit page.
    switches: bool,
}

impl FieldValue {
    /// `text` as a read page shows it: dropped, and so wiped, when `hidden`, not `shown` and not empty.
    #[must_use]
    pub fn new(text: Secret, hidden: bool, shown: bool) -> Self {
        Self::shown_as(text, hidden, shown, false)
    }

    /// `text` as the New or Edit page shows it, where its hidden flag can switch; dropped as `new` drops it, its count
    /// kept.
    #[must_use]
    pub fn edit(text: Secret, hidden: bool, shown: bool) -> Self {
        Self::shown_as(text, hidden, shown, true)
    }

    /// A value a read page masks, built from no copy of its text.
    #[must_use]
    pub const fn masked() -> Self {
        Self {
            text: None,
            count: None,
            hidden: true,
            shown: false,
            switches: false,
        }
    }

    /// A value of `count` characters the New or Edit page masks, built from no copy of its text.
    #[must_use]
    pub const fn masked_edit(count: usize) -> Self {
        Self {
            text: None,
            count: Some(Redacted::new(count)),
            hidden: true,
            shown: false,
            switches: true,
        }
    }

    /// `text` kept unless it is `hidden`, not `shown` and not empty; then a page that `switches` keeps its count.
    fn shown_as(text: Secret, hidden: bool, shown: bool, switches: bool) -> Self {
        let masked = hidden && !shown && !text.expose().is_empty();
        Self {
            count: (masked && switches).then(|| Redacted::new(text.expose().chars().count())),
            text: (!masked).then_some(text),
            hidden,
            shown: hidden && shown,
            switches,
        }
    }

    /// The text, or `None` while the mask shows.
    #[must_use]
    pub const fn text(&self) -> Option<&Secret> {
        self.text.as_ref()
    }

    /// The characters under the mask on New and Edit, one dot each; it tells a secret's length, so never log it.
    #[must_use]
    pub fn count(&self) -> Option<usize> {
        self.count.as_ref().map(|n| *n.expose())
    }

    /// The field is hidden, so it offers Show or Hide.
    #[must_use]
    pub const fn hidden(&self) -> bool {
        self.hidden
    }

    /// The field is hidden and shown now, so it offers Hide.
    #[must_use]
    pub const fn shown(&self) -> bool {
        self.shown
    }

    /// Its hidden flag can switch, as on the New and Edit pages.
    #[must_use]
    pub const fn switches(&self) -> bool {
        self.switches
    }
}

/// A field's line: its label and value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldLine {
    /// Its label.
    pub label: Label,
    /// Its value, or the mask.
    pub value: FieldValue,
    /// It offers Make one: a Password field on the New or Edit page.
    pub make: bool,
}
