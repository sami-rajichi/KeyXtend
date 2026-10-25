//! Loading the sealed blobs: a damaged one starts empty and tells once, and a locked store locks the run. One that
//! cannot be loaded, or opens but does not read, starts empty and is never written over that run; the unread one is
//! also set aside once.

use super::Actor;
use super::store::{Blob, aside_blob, emails_blob, learned_blob};
use crate::email::Emails;
use crate::learn::Learner;
use crate::pack::Pack;
use kx_module_api::keys;
use kx_module_api::ports::{BlobName, SealedError};
use std::fmt::Display;

/// Why a learned file of another language is not read, as logged.
const OTHER_LANGUAGE: &str = "the file names another language";

impl Actor {
    /// Loads the known emails; damaged or unread ones start empty.
    pub(super) fn load_emails(&mut self) {
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

    /// Loads the language in front's learner and merges its strays into `seed`; a damaged or unread one, or one of
    /// another language, starts empty.
    pub(super) fn load_learner(&mut self, seed: Option<&Pack<'_>>) {
        self.learner = None;
        let Some(name) = learned_blob(&self.iso) else {
            return;
        };
        let cfg = self.learn_cfg;
        let read = self.load(&name, Blob::Learned).map(|b| {
            let learner = Learner::from_bytes(&b, cfg);
            (b, learner)
        });
        let learner = match read {
            Some((_, Ok(learner))) if learner.iso() == self.iso => Some(learner),
            Some((bytes, Ok(_))) => self.unread(&name, Blob::Learned, &bytes, &OTHER_LANGUAGE),
            Some((bytes, Err(error))) => self.unread(&name, Blob::Learned, &bytes, &error),
            None => None,
        };
        let mut learner = learner.or_else(|| Learner::new(&self.iso, cfg).ok());
        if let Some(learner) = learner.as_mut() {
            learner.reconcile(seed);
        }
        self.learner = learner;
    }

    /// Blob `name` opened as `bytes` but did not read, for `why`: it is logged, told once, never written over this
    /// run, and copied to its aside blob unless an earlier copy is there; nothing is read from it.
    fn unread<T>(
        &mut self,
        name: &BlobName,
        blob: Blob,
        bytes: &[u8],
        why: &dyn Display,
    ) -> Option<T> {
        tracing::warn!(%why, ?blob, "predict blob not read");
        self.say_once(keys::WORDS_DAMAGED);
        self.saves.hold(blob);
        let aside = aside_blob(name);
        let store = &self.ports.store;
        let free = aside.filter(|a| matches!(store.load(a), Ok(None)));
        if let Some(Err(error)) = free.map(|a| store.save(&a, bytes)) {
            tracing::warn!(%error, ?blob, "predict blob not set aside");
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
