//! The fake's live run: reads cut as the real adapter cuts them, and end checks as the test scripted them.

use super::Shared;
use crate::lock;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    FieldConfig, FieldEnds, FieldError, FieldKind, FieldRead, FieldRun, last_words,
};
use kx_platform::field::after_first;
use std::sync::Arc;

/// The end of `text` one read of at most `chars` characters takes: all of it when it fits, else its last `chars`
/// characters without the first word they may split, as on the real adapter.
fn capped(text: &str, chars: usize) -> &str {
    let skip = text.chars().count().saturating_sub(chars);
    if skip == 0 {
        return text;
    }
    let at = text
        .char_indices()
        .nth(skip)
        .map_or(text.len(), |(at, _)| at);
    after_first(&text[at..])
}

/// A live fake watch; dropping it stops the watch and its tells.
pub(super) struct FakeRun {
    pub(super) shared: Arc<Shared>,
    /// The watch's config, whose `read_chars` caps each read.
    pub(super) cfg: FieldConfig,
}

impl FieldRun for FakeRun {
    fn kind(&self) -> FieldKind {
        lock(&self.shared.log).kind
    }

    fn before_cursor(&mut self, max_words: u8) -> Result<FieldRead, FieldError> {
        let mut log = lock(&self.shared.log);
        log.before_calls += 1;
        let told = FieldRead {
            kind: log.kind,
            text: None,
        };
        if !log.readable() {
            return Ok(told);
        }
        if let Some(error) = log.read_fail {
            return Err(error);
        }
        let kind = log.seen();
        let text = log.before.as_deref().filter(|_| kind.reads());
        let text = text
            .map(|t| last_words(capped(t, usize::from(self.cfg.read_chars)), max_words).to_owned());
        Ok(FieldRead {
            kind,
            text: text.map(Redacted::new),
        })
    }

    fn ends_with(&mut self, text: &Redacted<String>) -> FieldEnds {
        let mut log = lock(&self.shared.log);
        log.ends_calls += 1;
        log.last_ends = Some(text.clone());
        let unsure = log.read_fail.is_some() || !self.cfg.fits(text.expose());
        if !log.readable() || unsure {
            let ends = (!log.readable()).then_some(false);
            return FieldEnds::new(log.kind, ends);
        }
        let kind = log.seen();
        match (kind.reads(), log.selected.is_empty()) {
            (false, _) => FieldEnds::new(kind, Some(false)),
            (true, false) => FieldEnds::selection(kind),
            (true, true) => FieldEnds::new(kind, log.ends),
        }
    }
}

impl Drop for FakeRun {
    fn drop(&mut self) {
        let mut log = lock(&self.shared.log);
        log.stops += 1;
        let idle = log.watches == log.stops;
        drop(log);
        if idle {
            let gone = lock(&self.shared.tell).take();
            drop(gone);
        }
    }
}
