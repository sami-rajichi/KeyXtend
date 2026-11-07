//! The notebook's emails (Quick-fill's `VaultEmails`, ADR-0038): kept in memory only and wiped as they go, never
//! stored or learned, and offered as known emails with a key icon.

use std::fmt;

use kx_module_api::text::Casing;
use kx_module_api::{Redacted, Secret, Zeroizing};

use super::looks_like;

/// The notebook's emails, in the order the vault told them; the default holds none.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Saved {
    /// Each email and its lower-case key, both wiped when dropped.
    list: Vec<(Secret, Secret)>,
}

impl fmt::Debug for Saved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Saved")
            .field("count", &self.list.len())
            .finish()
    }
}

impl Saved {
    /// The values of `emails` that look like emails, once each in any case, the first spelling kept; the others are
    /// wiped.
    #[must_use]
    pub fn new(emails: Vec<Secret>) -> Self {
        let mut list: Vec<(Secret, Secret)> = Vec::new();
        for email in emails {
            let key = Zeroizing::new(Casing::root().lower(email.expose()));
            let fresh = !list.iter().any(|(_, k)| **k.expose() == *key);
            if fresh && looks_like(email.expose()) {
                list.push((email, Redacted::new(key)));
            }
        }
        Self { list }
    }

    /// Each email's lower-case key and its spelling, in order.
    pub(super) fn keyed(&self) -> impl Iterator<Item = (&str, &str)> {
        let shown = self.list.iter();
        shown.map(|(text, key)| (key.expose().as_str(), text.expose().as_str()))
    }
}

#[cfg(test)]
pub(super) mod tests;
