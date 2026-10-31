//! Follow-the-clear and skipped apps (spec §7 "Skipped copies"): a clear by the app that made the last kept copy,
//! within `clear_follow_s`, deletes that entry for good, as password managers intend; copies from `skip_apps` are
//! never kept.

use std::time::Duration;

use kx_module_api::ports::AppRef;
use kx_module_api::{Mono, Redacted};

/// The last kept entry and who copied it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Last {
    /// The entry.
    id: u64,
    /// The id of the app that copied it.
    app: Redacted<String>,
    /// When it was kept, in monotonic time.
    at: Mono,
}

/// Follows the last kept entry for a clear, and knows the skipped apps; its `Debug` hides app ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Follow {
    /// How long after a copy its app's clear deletes it (`clear_follow_s`).
    window: Duration,
    /// The ids of the apps whose copies are never kept (`skip_apps`).
    skip: Vec<Redacted<String>>,
    /// The last kept entry, while it can still be followed.
    last: Option<Last>,
}

impl Follow {
    /// Follows clears for `window` after a copy, and skips copies from the apps `skip_apps` names by id.
    #[must_use]
    pub fn new(window: Duration, skip_apps: Vec<String>) -> Self {
        let skip = skip_apps.into_iter().map(Redacted::new).collect();
        Self {
            window,
            skip,
            last: None,
        }
    }

    /// True when a copy from `app` is never kept: its id matches a skipped app's, ignoring case. The caller drops the
    /// copy's parts at once.
    #[must_use]
    pub fn skips(&self, app: Option<&AppRef>) -> bool {
        app.is_some_and(|a| self.skip.iter().any(|s| same_id(s.expose(), &a.id)))
    }

    /// Notes that entry `id`, copied by `app`, was kept or moved up at `at`; a copy by an unknown app is not followed.
    pub fn kept(&mut self, id: u64, app: Option<&AppRef>, at: Mono) {
        self.last = app.map(|a| Last {
            id,
            app: Redacted::new(a.id.clone()),
            at,
        });
    }

    /// The entry to delete for good when `app` clears the clipboard at `at`: the last kept one, when the same known
    /// app copied it no more than the window before.
    pub fn cleared(&mut self, app: Option<&AppRef>, at: Mono) -> Option<u64> {
        let last = self.last.as_ref()?;
        let same = app.is_some_and(|a| same_id(last.app.expose(), &a.id));
        let id = last.id;
        if same && at.saturating_since(last.at) <= self.window {
            self.last = None;
            Some(id)
        } else {
            None
        }
    }
}

/// True when app ids `a` and `b` match, ignoring case.
fn same_id(a: &str, b: &str) -> bool {
    a.chars()
        .flat_map(char::to_lowercase)
        .eq(b.chars().flat_map(char::to_lowercase))
}

#[cfg(test)]
mod tests;
