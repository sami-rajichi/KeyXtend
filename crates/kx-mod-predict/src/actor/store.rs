//! The sealed blobs: the language in front's learned words and the known emails. Each saves `save_s` after its first
//! unsaved change and at stop, a locked store saves nothing that run, and a new day purges; while learning is paused,
//! neither purges nor timed saves write.

use super::Actor;
use kx_module_api::ports::{BlobName, SealedError, fits_iso};
use kx_module_api::{Mono, keys};
use std::time::Duration;

/// The blob of the known emails, one for every language.
pub const EMAILS_BLOB: &str = "predict-emails";
/// What each language's learned-words blob is named after: then the language code.
const LEARNED_PREFIX: &str = "predict-";
/// What follows a blob's name in the name of its set-aside copy.
pub const ASIDE_SUFFIX: &str = "-aside";

/// The blob of language `iso`'s learned words; `None` for a code no pack could carry.
#[must_use]
pub fn learned_blob(iso: &str) -> Option<BlobName> {
    fits_iso(iso)
        .then(|| BlobName::new(&format!("{LEARNED_PREFIX}{iso}")).ok())
        .flatten()
}

/// The blob of the known emails; a test keeps its name sound.
#[must_use]
pub fn emails_blob() -> Option<BlobName> {
    BlobName::new(EMAILS_BLOB).ok()
}

/// The blob that keeps a copy of `name`'s bytes when they open but do not read: its name, then `ASIDE_SUFFIX`.
#[must_use]
pub fn aside_blob(name: &BlobName) -> Option<BlobName> {
    BlobName::new(&format!("{}{ASIDE_SUFFIX}", name.as_str())).ok()
}

/// One of the two blobs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Blob {
    /// The language in front's learned words.
    Learned,
    /// The known emails.
    Emails,
}

/// When each blob changed first since its last save, and what keeps it from saving.
#[derive(Debug, Default)]
pub(super) struct Saves {
    /// From a first unsaved change to its save.
    every: Duration,
    learned: Option<Mono>,
    emails: Option<Mono>,
    /// The OS key was refused: nothing saves this run.
    locked: bool,
    /// The blob could not be read, so it is never written over this run.
    held_learned: bool,
    held_emails: bool,
    /// The day of the last purge.
    pub(super) day: Option<u32>,
}

impl Saves {
    /// Saves due `every` after a first change.
    pub(super) fn new(every: Duration) -> Self {
        Self {
            every,
            ..Self::default()
        }
    }

    /// How long from `now` until the next save is due; `None` when nothing waits.
    pub(super) fn next(&self, now: Mono) -> Option<Duration> {
        let due =
            |first: Option<Mono>| first.map(|t| t.saturating_add(self.every).saturating_since(now));
        [due(self.learned), due(self.emails)]
            .into_iter()
            .flatten()
            .min()
    }

    /// The first unsaved change of `blob`.
    fn first(&mut self, blob: Blob) -> &mut Option<Mono> {
        match blob {
            Blob::Learned => &mut self.learned,
            Blob::Emails => &mut self.emails,
        }
    }

    /// True when `blob` may not be written this run.
    fn held(&self, blob: Blob) -> bool {
        self.locked
            || match blob {
                Blob::Learned => self.held_learned,
                Blob::Emails => self.held_emails,
            }
    }

    /// `blob` is never written over this run.
    pub(super) fn hold(&mut self, blob: Blob) {
        match blob {
            Blob::Learned => self.held_learned = true,
            Blob::Emails => self.held_emails = true,
        }
    }
}

impl Actor {
    /// `blob` changed now; its save is due `save_s` after its first unsaved change.
    pub(super) fn changed(&mut self, blob: Blob) {
        if !self.saves.held(blob) {
            let now = self.ports.clock.now();
            self.saves.first(blob).get_or_insert(now);
        }
    }

    /// Purges on a new day, then saves each blob whose time has come; nothing while learning is paused.
    pub fn tick(&mut self) {
        if !self.learns() {
            return;
        }
        self.new_day();
        let now = self.ports.clock.now();
        for blob in [Blob::Learned, Blob::Emails] {
            if self
                .saves
                .first(blob)
                .is_some_and(|t| t.saturating_add(self.saves.every) <= now)
            {
                self.save(blob);
            }
        }
    }

    /// Saves `blob` now when it changed; a refused save tries again `save_s` later, a locked one never this run.
    pub(super) fn save(&mut self, blob: Blob) {
        if self.saves.first(blob).take().is_none() || self.saves.held(blob) {
            return;
        }
        let saving = match blob {
            Blob::Learned => self
                .learner
                .as_ref()
                .and_then(|l| Some((learned_blob(&self.iso)?, l.to_bytes()))),
            Blob::Emails => emails_blob().map(|name| (name, self.emails.to_bytes())),
        };
        let Some((name, bytes)) = saving else {
            return;
        };
        match self.ports.store.save(&name, &bytes) {
            Ok(()) => self.stats.saves += 1,
            Err(SealedError::Locked) => self.lock(),
            Err(error) => {
                tracing::warn!(%error, ?blob, "predict blob not saved");
                self.stats.unsaved += 1;
                *self.saves.first(blob) = Some(self.ports.clock.now());
            }
        }
    }

    /// The language changes: the learner's timer and hold end with it.
    pub(super) fn leave(&mut self) {
        self.saves.learned = None;
        self.saves.held_learned = false;
        self.learner = None;
    }

    /// The OS key was refused: nothing saves this run, and the user hears it once.
    pub(super) fn lock(&mut self) {
        self.saves.locked = true;
        (self.saves.learned, self.saves.emails) = (None, None);
        self.say_once(keys::WORDS_LOCKED);
    }

    /// On a new day, drops what went unused for more than `forget_days`; nothing while learning is paused.
    pub(super) fn new_day(&mut self) {
        if !self.learns() {
            return;
        }
        let today = self.ports.days.today();
        if self.saves.day.replace(today) == Some(today) {
            return;
        }
        let Some(learner) = self.learner.as_mut() else {
            return;
        };
        let before = learner.clone();
        learner.purge(today);
        if *learner != before {
            self.changed(Blob::Learned);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_blobs_are_named_soundly_and_never_for_a_bad_code() {
        assert_eq!(
            emails_blob().map(|n| n.as_str().to_owned()).as_deref(),
            Some(EMAILS_BLOB)
        );
        let en = learned_blob("en").map(|n| n.as_str().to_owned());
        assert_eq!(en.as_deref(), Some("predict-en"));
        assert!(learned_blob("").is_none() && learned_blob("../x").is_none());
        assert!(learned_blob("fil").is_some(), "a three-letter code");
        let fil = learned_blob("fil").unwrap();
        for name in [emails_blob().unwrap(), fil] {
            let aside = aside_blob(&name).map(|n| n.as_str().to_owned());
            assert_eq!(aside, Some(format!("{}{ASIDE_SUFFIX}", name.as_str())));
        }
    }

    #[test]
    fn the_next_save_is_the_earliest_due() {
        let mut saves = Saves::new(Duration::from_secs(30));
        let at = |s: u64| Mono::from_us(s * 1_000_000);
        assert_eq!(saves.next(at(0)), None);
        saves.emails = Some(at(10));
        saves.learned = Some(at(5));
        assert_eq!(saves.next(at(20)), Some(Duration::from_secs(15)));
        assert_eq!(saves.next(at(99)), Some(Duration::ZERO), "overdue");
    }
}
