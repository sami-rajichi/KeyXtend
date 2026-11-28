//! Snip on the hub (P12): it keeps the newest `SnipView` with its pictures, wakes the Snip object once per read, and
//! sends the clicks, the theme's fit, the flattened picture and each screen's work area through a handle limited to the
//! `ui` module's capabilities. Nothing here logs a pixel, a path or a text.

pub mod acts;
mod edit;
mod frozen;
mod opened;
mod pics;
mod place;
mod shield;
mod view;

use super::unread::Unread;
use super::{Hub, NULL, bus, wiped};
use crate::ui_module::MANIFEST;
use acts::{click_of, fit_of, keep_of, round_of, silent};
use frozen::Listen;
use kx_module_api::ports::{Shot, SnipAct, SnipAside, SnipClick, SnipPage, SnipStage, SnipView};
use kx_module_api::{Bus, EventError, Subscription, Zeroizing};
pub use opened::{SnipCounted, SnipOpened};
use pics::Pics;
#[cfg(feature = "qt")]
pub(crate) use pics::SHOTS;
pub use place::EditorAt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

/// What changed for the Snip object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnipWake {
    /// A new view or a changed shield: call `snip_woken`, then read `snip_json`; no other comes until then.
    View,
}

/// Hands a wake to the Snip object on the Qt thread; it must not block.
pub type SnipWaker = Box<dyn Fn(SnipWake) + Send + Sync>;

/// Snip's side of the hub: the view, whether its wake waits for a read, and the object's waker.
#[derive(Default)]
pub(super) struct Snip {
    state: Mutex<State>,
    view: Unread,
    waker: Mutex<Option<Arc<SnipWaker>>>,
}

/// The view shown, its pictures, and the handle that sends the clicks.
#[derive(Default)]
struct State {
    /// The bus limited to the `ui` module's capabilities, once `listen` has it.
    bus: Option<Bus>,
    /// The newest view.
    page: Option<Arc<SnipPage>>,
    /// Its pictures, for the provider.
    pics: Pics,
    /// A snip started and the module has not heard the screens since.
    started: bool,
    /// The capture windows whose shield was refused.
    bare: Vec<&'static str>,
    /// The hub itself, which the hand's Esc watch holds weakly.
    me: Weak<Hub>,
    /// The hand's Esc watch.
    esc: Listen,
    /// The frozen screens' title as QML gave it last, which they lie low by.
    screen: String,
    /// A refused order was warned of for these frozen screens.
    warned: bool,
}

impl Hub {
    /// Hears the views and the keyboard's aside through a handle limited to the `ui` module's capabilities, and keeps
    /// it for the clicks.
    pub(super) fn hear_snip(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        let gated = bus.restricted(MANIFEST.capabilities);
        let heard = vec![
            self.hear(&gated, |hub, v: &SnipView| hub.snip_view(&v.0))?,
            self.hear(&gated, |hub, a: &SnipAside| hub.step_aside(*a))?,
        ];
        gated.check::<SnipAct>()?;
        let mut s = self.snip();
        s.bus = Some(gated);
        s.me = Arc::downgrade(self);
        Ok(heard)
    }

    /// Snip's windows as JSON for QML, or `null` before any view; reading it lets the next change wake again. It holds
    /// typed text and may name a folder, so it is wiped on drop; never log it.
    #[must_use]
    pub fn snip_json(&self) -> Zeroizing<String> {
        self.snip.view.read();
        let (page, generation, hide) = {
            let s = self.snip();
            let Some(page) = s.page.clone() else {
                return Zeroizing::new(NULL.to_owned());
            };
            let hide = shield::hidden(&page, &s.bare);
            (page, s.pics.generation(), hide)
        };
        wiped::json(&view::of(&page, generation, hide))
    }

    /// The Qt thread's work before QML reads a new view: at a snip's start, the module hears each screen's work area,
    /// then `Bare` while a capture window's shield was refused; and the hand's Esc is heard while frozen screens show.
    pub fn snip_woken(&self) {
        let (started, bare) = {
            let mut s = self.snip();
            (std::mem::take(&mut s.started), !s.bare.is_empty())
        };
        if started {
            self.snip_screens();
            if bare {
                self.snip_now(SnipClick::Bare);
            }
        }
        self.snip_listen();
    }

    /// A screen came, went or changed: the module hears each screen's work area. Call it on the Qt thread.
    pub fn snip_screens(&self) {
        let screens = self.ports.screens.work_areas();
        self.snip_now(SnipClick::Screens(screens));
    }

    /// Sends the click `kind` names (one of `acts`) with `id` and `arg` on the view of round `round`, sounding as a key
    /// does but for a caret. A bad click warns and a late one logs at debug, and neither sends or sounds; call it on
    /// the Qt thread.
    pub fn snip_act(&self, kind: &str, id: &str, arg: i32, round: i64) {
        if self.snip_page().is_none() {
            tracing::debug!("a late Snip click was dropped");
            return;
        }
        // The kind, id and argument are never logged: a caret's tells the typed text's length.
        let Ok((click, round)) = click_of(kind, id, arg).and_then(|c| Ok((c, round_of(round)?)))
        else {
            tracing::warn!("QML sent an unknown Snip click or a bad argument");
            return;
        };
        if !silent(kind) {
            self.tapped();
        }
        self.snip_send(&SnipAct { round, click });
    }

    /// The editor's picture flattened at its own pixels for `keep` ("copy" or "save"), as a key does; the pixels are
    /// shared on, copied only to cut a grab's extra pixels, and never logged.
    pub fn snip_flat(&self, keep: &str, shot: Shot, round: i64) {
        let Ok((keep, round)) = keep_of(keep).and_then(|k| Ok((k, round_of(round)?))) else {
            tracing::warn!("QML sent a flattened picture for no Copy or Save");
            return;
        };
        self.tapped();
        let shot = pics::kept(shot, self.snip_page().as_deref());
        let click = SnipClick::Flat { keep, shot };
        self.snip_send(&SnipAct { round, click });
    }

    /// The theme's sizes, as JSON, which the module places and steps by; false when they do not read. Call it whenever
    /// an editor opens, as each snip's scale may differ.
    pub fn snip_fit(&self, json: &str) -> bool {
        let Ok(fit) = fit_of(json) else {
            tracing::warn!("QML sent a Snip fit that does not read");
            return false;
        };
        self.snip_now(SnipClick::Fit(fit));
        true
    }

    /// Registers the Snip object's waker and hands it a wake QML has not read; a new one replaces the last, as the
    /// app's root may be made again.
    pub fn set_snip_waker(&self, waker: SnipWaker) {
        let replaced = self.snip_waker().replace(Arc::new(waker)).is_some();
        if replaced {
            tracing::debug!("a new Snip object took the Snip wakes");
        }
        if self.snip.view.waits() {
            self.snip_wake(SnipWake::View);
        }
    }

    /// The pixels of picture `id`, `<generation>/<place>`, shared, while the view that holds it is kept.
    #[must_use]
    pub fn snip_picture(&self, id: &str) -> Option<Shot> {
        self.snip().pics.get(id)
    }

    /// The newest view.
    fn snip_page(&self) -> Option<Arc<SnipPage>> {
        self.snip().page.clone()
    }

    /// Sends `click` with the newest view's round, or 0 before any; the module takes `Aside` only for the round told
    /// with its step aside or a newer one, and every other such click on any view.
    pub(super) fn snip_now(&self, click: SnipClick) {
        let round = self.snip_page().map_or(0, |p| p.round);
        self.snip_send(&SnipAct { round, click });
    }

    /// True while a snip runs: the newest view shows something.
    fn snip_live(&self) -> bool {
        self.snip_page()
            .is_some_and(|p| !matches!(p.stage, SnipStage::Idle))
    }

    /// Keeps `page` and its pictures and wakes; a page after an idle one starts a snip. The page it replaces goes after
    /// the lock, with its pixels, and a countdown second logs at debug off the Qt thread, as `snip_counted` does on it.
    fn snip_view(&self, page: &Arc<SnipPage>) {
        if let SnipStage::Count(c) = &page.stage {
            tracing::debug!(left_s = c.left_s, "a countdown second came");
        }
        let gone = {
            let mut s = self.snip();
            let idle = |p: &SnipPage| matches!(p.stage, SnipStage::Idle);
            s.started |= s.page.as_deref().is_none_or(idle) && !idle(page);
            s.pics.keep(page);
            s.page.replace(Arc::clone(page))
        };
        drop(gone);
        self.snip_view_woken();
    }

    /// Sends `event` through the limited handle, outside the lock; nothing goes before `listen`.
    fn snip_send(&self, act: &SnipAct) {
        let handle = self.snip().bus.clone();
        bus::send(handle, act, "a Snip click was not sent");
    }

    /// Wakes the object unless a wake already waits for its read.
    fn snip_view_woken(&self) {
        if self.snip.view.arm() {
            self.snip_wake(SnipWake::View);
        }
    }

    /// Calls the object's waker, if any, outside the lock.
    fn snip_wake(&self, what: SnipWake) {
        let waker = self.snip_waker().clone();
        if let Some(waker) = waker {
            waker(what);
        }
    }

    fn snip(&self) -> MutexGuard<'_, State> {
        self.snip
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn snip_waker(&self) -> MutexGuard<'_, Option<Arc<SnipWaker>>> {
        self.snip
            .waker
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
