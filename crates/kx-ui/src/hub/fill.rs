//! The Quick-fill panel on the hub (ADR-0038): while the vault's panel is open it keeps the newest `FillView` and
//! wakes the panel's object once per read, and sends its clicks, the ui's rows and the label names through a handle
//! limited to the `ui` module's capabilities. Nothing here logs a name, a value, a search or a typed password.

pub mod acts;
mod view;

use super::unread::Unread;
use super::{Hub, NULL, bus, json};
use crate::ui_module::MANIFEST;
use acts::{CARET, act_of, silent};
use kx_module_api::ports::{FillAct, FillAsk, FillClick, FillLive, FillNames, FillPanel, FillView};
use kx_module_api::{Bus, EventError, Subscription};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Hands a wake to the panel's object on the Qt thread; it must not block.
pub type FillWaker = Box<dyn Fn() + Send + Sync>;

/// The panel's side of the hub: what it shows, whether a wake waits for the object's read, and its waker.
#[derive(Default)]
pub(super) struct Fill {
    state: Mutex<State>,
    unread: Unread,
    waker: Mutex<Option<Arc<FillWaker>>>,
}

/// What the panel shows, and the handle that sends its clicks.
#[derive(Default)]
struct State {
    /// The bus limited to the `ui` module's capabilities, once `listen` has it.
    bus: Option<Bus>,
    /// The vault's panel is open, as its last `FillPanel` said.
    open: bool,
    /// The lowest round a new view may carry; it goes back to 0 as the panel opens or closes, as each open counts anew.
    floor: u32,
    /// The newest view heard while the panel was open.
    view: Option<FillView>,
}

impl Hub {
    /// Hears the vault's state, the panel's and its views through a handle limited to the `ui` module's capabilities,
    /// keeps that handle for the clicks, the rows and the names, and asks the vault what it missed.
    pub(super) fn hear_fill(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        let gated = bus.restricted(MANIFEST.capabilities);
        let heard = vec![
            self.hear(&gated, |hub, p: &FillPanel| hub.fill_panel(p.open))?,
            self.hear(&gated, |hub, v: &FillView| hub.fill_view(v.clone()))?,
            self.hear(&gated, |hub, l: &FillLive| hub.fill_live(l.live))?,
        ];
        gated.check::<FillAct>()?;
        gated.check::<FillNames>()?;
        gated.publish(&FillAsk)?;
        self.fill().bus = Some(gated);
        Ok(heard)
    }

    /// The panel as JSON for QML, every text worded, or `null` while it is closed or has no view yet; reading it lets
    /// the next change wake again. It holds names and shown values, so never log it.
    #[must_use]
    pub fn fill_json(&self) -> String {
        self.fill.unread.read();
        let view = self.fill().view.clone();
        view.map_or_else(|| NULL.to_owned(), |v| json(&view::of(&v.0, &self.i18n())))
    }

    /// Sends the click `kind` names (one of `acts`) with `id` and `arg`, on the view of round `round`, and sounds as a
    /// key does, but for a caret, whose line sounded, and the wheel. It goes only while a view is kept, with `round` as
    /// given, since the vault drops an older round's click on a row or a field; a bad click sends nothing and warns.
    /// Call it on the Qt thread.
    pub fn fill_act(&self, kind: &str, id: &str, arg: i32, round: i64) {
        if !silent(kind) {
            self.tapped();
        }
        if self.fill().view.is_none() {
            tracing::debug!("a late Quick-fill click was dropped");
            return;
        }
        // The kind and id QML sent are never logged, so a wrong one can never carry what the user typed.
        // A caret's number tells a hidden line's length, so it is never logged.
        if let Ok(act) = act_of(kind, id, arg, round) {
            self.send_fill(act);
        } else if kind == CARET {
            tracing::warn!("QML sent a bad Quick-fill caret click");
        } else {
            tracing::warn!(
                arg,
                "QML sent an unknown Quick-fill click or a bad argument"
            );
        }
    }

    /// Registers the `QuickFill` object's waker and hands it a wake QML has not read; a new one replaces the last, as
    /// the app's root may be made again.
    pub fn set_fill_waker(&self, waker: FillWaker) {
        let replaced = self.fill_waker().replace(Arc::new(waker)).is_some();
        if replaced {
            tracing::debug!("a new Quick-fill object took the panel's wakes");
        }
        if self.fill.unread.waits() {
            self.fill_wake();
        }
    }

    /// At the `ui` module's start: the vault hears the rows and the label names, and a kept view is worded again.
    pub(super) fn fill_start(&self) {
        self.fill_names();
        self.fill_rows();
        self.fill_reword();
    }

    /// The keyboard's language changed: the vault hears the new label names, and a kept view is worded again.
    pub(super) fn fill_relang(&self) {
        self.fill_names();
        self.fill_reword();
    }

    /// A pad page tap turns the open panel's page; true when it did.
    pub(super) fn fill_pad(&self, back: bool) -> bool {
        let round = {
            let s = self.fill();
            if !s.open {
                return false;
            }
            s.view.as_ref().map_or(0, |v| v.0.round)
        };
        let click = FillClick::Page { back };
        self.send_fill(FillAct { round, click });
        true
    }

    /// Tells the vault the rows a page holds, `[ui] page_rows`; it keeps them, open or not, and draws only after them.
    fn fill_rows(&self) {
        let rows = self.lock().settings.page_rows;
        let rows = FillClick::Rows(u8::try_from(rows).unwrap_or(u8::MAX));
        let round = self.fill().view.as_ref().map_or(0, |v| v.0.round);
        self.send_fill(FillAct { round, click: rows });
    }

    /// Tells the vault each table label's text in the keyboard's language, which its search and By label match.
    fn fill_names(&self) {
        let names = FillNames(self.i18n().fill_labels());
        let handle = self.fill().bus.clone();
        bus::send(handle, &names, "the Quick-fill label names were not sent");
    }

    /// Wakes the panel's object to word a kept view again.
    fn fill_reword(&self) {
        if self.fill().view.is_some() {
            self.fill_woken();
        }
    }

    /// Takes the panel's state: an open one hears the names and rows, a closing one drops its view and wakes once, and
    /// an open sent again, as the vault answers `FillAsk`, keeps the view and its floor.
    fn fill_panel(&self, open: bool) {
        let (was, gone) = {
            let mut s = self.fill();
            let was = std::mem::replace(&mut s.open, open);
            if was != open {
                s.floor = 0;
            }
            (was, if open { None } else { s.view.take() })
        };
        drop(gone);
        if open {
            self.fill_names();
            self.fill_rows();
        } else if was {
            self.fill_woken();
        }
    }

    /// Keeps `view` while the panel is open and wakes; one heard while closed, or older than the kept one, is late.
    fn fill_view(&self, view: FillView) {
        let last = {
            let mut s = self.fill();
            if !s.open || view.0.round < s.floor {
                return;
            }
            s.floor = view.0.round;
            s.view.replace(view)
        };
        drop(last);
        self.fill_woken();
    }

    /// Wakes the panel's object unless a wake already waits for its read.
    fn fill_woken(&self) {
        if self.fill.unread.arm() {
            self.fill_wake();
        }
    }

    /// Calls the panel's object's waker, if any, outside the lock.
    fn fill_wake(&self) {
        let waker = self.fill_waker().clone();
        if let Some(waker) = waker {
            waker();
        }
    }

    /// Sends `act` through the limited handle, outside the lock; nothing goes before `listen`.
    fn send_fill(&self, act: FillAct) {
        let handle = self.fill().bus.clone();
        bus::send(handle, &act, "a Quick-fill click was not sent");
    }

    fn fill(&self) -> MutexGuard<'_, State> {
        self.fill
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn fill_waker(&self) -> MutexGuard<'_, Option<Arc<FillWaker>>> {
        self.fill
            .waker
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
