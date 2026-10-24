//! Fake word packs: the test adds each language's bytes, and the fake counts the opens.

use crate::lock;
use kx_module_api::ports::{PackError, PackView, WordPacks};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// What the opens did, and what the test set for them.
#[derive(Default)]
struct Log {
    packs: BTreeMap<String, Arc<[u8]>>,
    fail: bool,
    opens: BTreeMap<String, usize>,
}

/// Word packs shared by their clones: each open gives a view over the bytes added for its language, or `None`.
#[derive(Clone, Default)]
pub struct FakeWordPacks {
    log: Arc<Mutex<Log>>,
}

impl FakeWordPacks {
    /// Packs with no language.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes `bytes` the pack of language `iso`, in place of any before; open views keep their bytes.
    pub fn add(&self, iso: &str, bytes: Vec<u8>) {
        lock(&self.log).packs.insert(iso.to_owned(), bytes.into());
    }

    /// Refuses the next open, whatever its language, with `Unreadable`.
    pub fn fail_open(&self) {
        lock(&self.log).fail = true;
    }

    /// How many opens asked for language `iso`, failed and missing ones too.
    #[must_use]
    pub fn opens(&self, iso: &str) -> usize {
        lock(&self.log).opens.get(iso).copied().unwrap_or(0)
    }
}

impl fmt::Debug for FakeWordPacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.log);
        f.debug_struct("FakeWordPacks")
            .field("languages", &log.packs.keys().collect::<Vec<_>>())
            .field("opens", &log.opens)
            .finish_non_exhaustive()
    }
}

impl WordPacks for FakeWordPacks {
    fn open(&self, iso: &str) -> Result<Option<Box<dyn PackView>>, PackError> {
        let mut log = lock(&self.log);
        *log.opens.entry(iso.to_owned()).or_default() += 1;
        if std::mem::take(&mut log.fail) {
            return Err(PackError::Unreadable);
        }
        let pack = log.packs.get(iso).map(|bytes| FakePack(Arc::clone(bytes)));
        Ok(pack.map(|p| Box::new(p) as Box<dyn PackView>))
    }
}

/// An open fake pack: a share of the bytes it was opened over.
struct FakePack(Arc<[u8]>);

impl PackView for FakePack {
    fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests;
