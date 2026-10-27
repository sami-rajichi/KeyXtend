//! Loading the sealed blobs: a damaged one starts empty and tells once, and a locked store locks the run. One that
//! opens but does not read is set aside like a damaged one, and one that cannot be loaded is never written that run.

use super::Actor;
use super::store::{Blob, aside_blob, emails_blob, learned_blob};
use crate::email::Emails;
use crate::learn::Learner;
use crate::pack::Pack;
use kx_module_api::keys;
use kx_module_api::ports::{BlobName, SealedError};
use std::fmt::Display;

/// Why a learned file of another language is not read, as logged; tests look for it.
pub const OTHER_LANGUAGE: &str = "the file names another language";

impl Actor {
    /// Loads the known emails once while prediction is on; damaged or unread ones start empty.
    pub(super) fn load_emails(&mut self) {
        if self.emails_loaded || !self.settings.on {
            return;
        }
        self.emails_loaded = true;
        let Some(name) = emails_blob() else {
            return;
        };
        let max = self.email_cfg.max_emails;
        let read = self.load(&name, Blob::Emails).map(|b| {
            let emails = Emails::from_bytes(&b, max);
            (b, emails)
        });
        let emails = match read {
            Some((_, Ok(emails))) => Some(emails),
            Some((bytes, Err(error))) => self.unread(&name, Blob::Emails, &bytes, &error),
            None => None,
        };
        self.emails = emails.unwrap_or_default();
    }

    /// Loads the language in front's learner, saved under the layout's code and learning in its words' code, and merges
    /// its strays into `seed`; words a failed save left come back first. A damaged or unread one, or one of another
    /// language, starts empty.
    pub(super) fn load_learner(&mut self, seed: Option<&Pack<'_>>) {
        self.learner = None;
        let Some(name) = learned_blob(&self.iso) else {
            return;
        };
        let cfg = self.learn_cfg;
        let left = self.saves.take_left(&name);
        let back = left.is_some();
        let read = left.or_else(|| self.load(&name, Blob::Learned)).map(|b| {
            let learner = Learner::from_bytes(&b, cfg);
            (b, learner)
        });
        let learner = match read {
            Some((_, Ok(learner))) if learner.iso() == self.code => Some(learner),
            Some((bytes, Ok(_))) => self.unread(&name, Blob::Learned, &bytes, &OTHER_LANGUAGE),
            Some((bytes, Err(error))) => self.unread(&name, Blob::Learned, &bytes, &error),
            None => None,
        };
        let mut learner = learner.or_else(|| Learner::new(&self.code, cfg).ok());
        if let Some(learner) = learner.as_mut() {
            learner.reconcile(seed);
        }
        self.learner = learner;
        if back {
            self.changed(Blob::Learned);
        }
    }

    /// Blob `name` opened as `bytes` but did not read, for `why` (plan edge 10): it is logged, told once and copied
    /// into its aside blob, the newest copy winning, so saves resume; a copy that fails holds the blob for the run.
    fn unread<T>(
        &mut self,
        name: &BlobName,
        blob: Blob,
        bytes: &[u8],
        why: &dyn Display,
    ) -> Option<T> {
        tracing::warn!(%why, ?blob, "predict blob not read");
        self.say_once(keys::WORDS_DAMAGED);
        let copied = aside_blob(name).map(|aside| self.ports.store.save(&aside, bytes));
        if let Some(Err(error)) = copied {
            tracing::warn!(%error, ?blob, "predict blob not set aside");
        }
        if !matches!(copied, Some(Ok(()))) {
            self.saves.hold(blob);
        }
        None
    }

    /// The bytes under `name`, or none; a damaged blob tells once, a locked store locks the run, and a blob that
    /// cannot be read is never written over this run.
    fn load(&mut self, name: &BlobName, blob: Blob) -> Option<Vec<u8>> {
        match self.ports.store.load(name) {
            Ok(bytes) => bytes,
            Err(SealedError::Damaged) => {
                self.say_once(keys::WORDS_DAMAGED);
                None
            }
            Err(SealedError::Locked) => {
                self.lock();
                None
            }
            Err(error) => {
                tracing::warn!(%error, ?blob, "predict blob not loaded");
                self.saves.hold(blob);
                None
            }
        }
    }
}
