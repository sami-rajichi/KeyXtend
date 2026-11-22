//! The tests' `Supply`: what each language gets, which a test changes as it goes.

use super::*;

/// What each language gets: its local model, then its cloud pick, if any; the services still switched on; and the
/// service Off each language's pick waits for.
pub(super) struct Book(
    pub(super) BTreeMap<String, Result<Ready, Lack>>,
    pub(super) BTreeMap<String, Net>,
    pub(super) BTreeSet<String>,
    pub(super) BTreeMap<String, String>,
);

impl Supply for Book {
    fn supply(&self, iso: &str) -> Result<Ready, Lack> {
        self.0.get(iso).cloned().unwrap_or(Err(Lack::Unknown))
    }

    fn cloud(&self, iso: &str) -> Option<Net> {
        self.1.get(iso).cloned()
    }

    fn on(&self, service: &str) -> bool {
        self.2.contains(service)
    }

    fn off(&self, iso: &str) -> Option<String> {
        self.3.get(iso).cloned()
    }
}
