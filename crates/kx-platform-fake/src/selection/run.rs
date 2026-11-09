//! The fake's live run: looks, `still`, reads and the clipboard answer as the test scripted them, and each call counts.

use super::Shared;
use crate::lock;
use kx_module_api::Redacted;
use kx_module_api::ports::same_box;
use kx_module_api::ports::{SelError, SelKind, SelRun, SelSpot};
use std::sync::Arc;

/// A live fake watch; dropping it stops the watch and its tells.
pub(super) struct FakeSelRun {
    pub(super) shared: Arc<Shared>,
}

impl SelRun for FakeSelRun {
    fn look(&mut self) -> Result<SelSpot, SelError> {
        let mut log = lock(&self.shared.log);
        log.calls.looks += 1;
        log.look_fail.map_or(Ok(log.spot), Err)
    }

    fn ours(&self) -> bool {
        lock(&self.shared.log).ours
    }

    fn still(&mut self, box_id: u64) -> bool {
        let mut log = lock(&self.shared.log);
        log.calls.stills += 1;
        if log.ours {
            return false;
        }
        let held = same_box(box_id, log.spot.box_id) && log.spot.selected;
        log.still.unwrap_or(held)
    }

    fn read(&mut self, box_id: u64, max_chars: u16) -> Result<Redacted<String>, SelError> {
        let mut log = lock(&self.shared.log);
        if log.ours || log.spot.kind == SelKind::Closed || !same_box(log.spot.box_id, box_id) {
            return Err(SelError::Closed);
        }
        log.calls.reads += 1;
        if let Some(error) = log.read_fail {
            return Err(error);
        }
        let text = if log.spot.selected {
            log.text.chars().take(usize::from(max_chars)).collect()
        } else {
            String::new()
        };
        Ok(Redacted::new(text))
    }

    fn clip_holds(&self) -> bool {
        let mut log = lock(&self.shared.log);
        log.calls.clips += 1;
        log.clip
    }
}

impl Drop for FakeSelRun {
    fn drop(&mut self) {
        let mut log = lock(&self.shared.log);
        log.calls.stops += 1;
        let idle = log.calls.watches == log.calls.stops;
        drop(log);
        if idle {
            let gone = lock(&self.shared.tell).take();
            drop(gone);
        }
    }
}
