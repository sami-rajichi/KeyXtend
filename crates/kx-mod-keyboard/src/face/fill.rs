//! Quick-fill on the face (spec §8.1, ADR-0038 decision 4): the fill key asks for its panel, and a fill
//! types on the sender's thread up to its first Tab gap, the rest at the UI's next call after it, so no thread ever
//! waits; too late, to another app or after an outside click, the rest never goes. As in `bus.rs`, nothing is sent
//! while the face's lock is held, and no value is ever logged.

use super::Face;
use kx_module_api::Bus;
use kx_module_api::ports::{FillOpen, FillType, FillTyped};
use std::sync::atomic::Ordering::SeqCst;

impl Face {
    /// The fill key: asks the vault module to open or close its panel while it runs, else does nothing.
    pub(super) fn fill_open(&self) {
        let bus = {
            let inner = self.lock();
            inner.vault_live().then(|| inner.bus())
        };
        if let Some(bus) = bus {
            // `FillOpen` needs no capability, so none is refused.
            let _ = bus.publish(&FillOpen);
        }
    }

    /// The vault module asks for fill `ask`: a waiting fill that is late or clicked away ends first with no, as no UI
    /// call may have come, then its parts go up to the first Tab gap, and the module hears the answer once done; then
    /// the bus hears the latches and the break. While a fill still waits inside its bound, `ask` answers no at once.
    pub(super) fn fill(&self, ask: &FillType) {
        let (ended, done, bus) = {
            let mut inner = self.lock();
            let clicks = self.clicks.load(SeqCst);
            let ended = inner.expire(clicks);
            (ended, inner.start_fill(ask, clicks), inner.bus())
        };
        if let Some(round) = ended {
            answer(&bus, round, false);
        }
        if let Some(ok) = done {
            answer(&bus, ask.round, ok);
        }
        self.pour();
        self.tell(false);
    }

    /// Sends a waiting fill's next parts once its gap has passed, at any call from the UI; a click outside our windows
    /// since it started, a part more than `fill_late_ms` past due, or another app in front, ends it unsent.
    pub(super) fn fill_due(&self) {
        let (done, bus) = {
            let mut inner = self.lock();
            let clicks = self.clicks.load(SeqCst);
            (inner.fill_due(clicks), inner.bus())
        };
        if let Some((round, ok)) = done {
            answer(&bus, round, ok);
        }
    }

    /// The module stopped: nothing more types, and in the same lock a waiting fill ends unsent with the answer no.
    pub(super) fn fill_stop(&self) {
        let (left, bus) = {
            let mut inner = self.lock();
            inner.set_running(false);
            (inner.drop_fill(), inner.bus())
        };
        if let Some(round) = left {
            answer(&bus, round, false);
        }
    }
}

/// Tells the vault module whether every part of fill `round` went out.
fn answer(bus: &Bus, round: u32, ok: bool) {
    // The module holds `Vault`, which the answer needs, or its start would have failed.
    let _ = bus.publish(&FillTyped { round, ok });
}
