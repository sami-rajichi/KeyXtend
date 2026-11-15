//! Voice typing on the hub (P11): it keeps the newest `VoiceView` and wakes the Voice object once per read, keeps the
//! sound level and wakes only the meter, places the bar, and sends the clicks, the rows and the panel's open through a
//! handle limited to the `ui` module's capabilities. Nothing here logs a text, a name or a path.

pub mod acts;
mod device;
mod lists;
mod options;
mod panel;
mod parts;
mod place;
mod sheets;
mod view;

use super::unread::Unread;
use super::{Hub, NULL, bus, json};
use crate::i18n::keys::VOICE_READ_TEXT;
use crate::ui_module::MANIFEST;
use crate::{I18n, spoken};
use acts::{Click, click_of, round_of, silent};
use kx_module_api::ports::{
    BarPlace, CompareSheet, LEVEL_FULL, VoiceAct, VoiceAsk, VoiceBar, VoiceClick, VoiceLevel,
    VoiceOpen, VoicePage, VoiceSheet, VoiceView,
};
use kx_module_api::{Bus, Event, EventError, Redacted, Subscription};
use kx_theme::sounds::Event as Sound;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// What changed for the Voice object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VoiceWake {
    /// A new view, or the keyboard's language changed: read `voice_json`; no other comes until then.
    View,
    /// A new sound level: read `voice_level`; no other comes until then.
    Level,
}

/// Hands a wake to the Voice object on the Qt thread; it must not block.
pub type VoiceWaker = Box<dyn Fn(VoiceWake) + Send + Sync>;

/// Voice typing's side of the hub: the view, the level, whether each wake waits for a read, and the object's waker.
#[derive(Default)]
pub(super) struct Voice {
    state: Mutex<State>,
    view: Unread,
    meter: Unread,
    level: AtomicU8,
    waker: Mutex<Option<Arc<VoiceWaker>>>,
}

/// The view shown and the handle that sends the clicks.
#[derive(Default)]
struct State {
    /// The bus limited to the `ui` module's capabilities, once `listen` has it.
    bus: Option<Bus>,
    /// The newest view.
    page: Option<Arc<VoicePage>>,
    /// The pointer is over the bar, as QML last said.
    over: bool,
}

impl Hub {
    /// Hears the views and the level through a handle limited to the `ui` module's capabilities, keeps it for the
    /// clicks, tells the module the rows a page holds and asks it what it missed.
    pub(super) fn hear_voice(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        let gated = bus.restricted(MANIFEST.capabilities);
        let heard = vec![
            self.hear(&gated, |hub, v: &VoiceView| hub.voice_view(&v.0))?,
            self.hear(&gated, |hub, l: &VoiceLevel| hub.voice_heard(l.level))?,
        ];
        gated.check::<VoiceAct>()?;
        gated.check::<VoiceOpen>()?;
        self.voice().bus = Some(gated.clone());
        self.voice_rows();
        gated.publish(&VoiceAsk)?;
        Ok(heard)
    }

    /// The bar and the panel as JSON for QML, every word in the keyboard's language, or `null` before any view;
    /// reading it lets the next view wake again. It holds the dictated text, so never log it.
    #[must_use]
    pub fn voice_json(&self) -> String {
        self.voice.view.read();
        let Some(page) = self.voice().page.clone() else {
            return NULL.to_owned();
        };
        let read = reads(&page).then(|| self.set_text()).flatten();
        json(&view::of(&page, &self.i18n(), read.as_deref()))
    }

    /// The sound level while the bar listens, 0 to 1, else 0; reading it lets the next level wake again.
    #[must_use]
    pub fn voice_level(&self) -> f64 {
        self.voice.meter.read();
        f64::from(self.voice.level.load(Ordering::SeqCst)) / f64::from(LEVEL_FULL)
    }

    /// Sends the click `kind` names (one of `acts`) with `id` and `arg`, on the view of round `round`, and sounds as a
    /// key does but for the pointer and the wheel. Nothing goes before a view; a bad click sends nothing and warns.
    /// Call it on the Qt thread.
    pub fn voice_act(&self, kind: &str, id: &str, arg: i32, round: i64) {
        if !silent(kind) {
            self.tapped();
        }
        if self.voice().page.is_none() {
            tracing::debug!("a late voice click was dropped");
            return;
        }
        // The kind and id QML sent are never logged; a wrong one could carry anything.
        let parsed = click_of(kind, id, arg).and_then(|c| round_of(round).map(|r| (c, r)));
        let Ok((click, round)) = parsed else {
            tracing::warn!(arg, "QML sent an unknown voice click or a bad argument");
            return;
        };
        let click = match click {
            Click::Gear => return self.voice_send(&VoiceOpen),
            Click::Compare => {
                VoiceClick::Compare(Redacted::new(self.set_text().unwrap_or_default()))
            }
            Click::Send(click) => click,
        };
        if let VoiceClick::Over(over) = click {
            self.voice().over = over;
        }
        self.voice_send(&VoiceAct { round, click });
    }

    /// The bar started listening (`start`) or stopped: plays the start or stop sound at the `[ui]` volume, unless the
    /// shown view has `[voice] sounds` off or no view came yet. Call it on the Qt thread.
    pub fn voice_chime(&self, start: bool) {
        let sounds = self.voice().page.as_ref().is_some_and(|p| p.sounds);
        if sounds {
            self.sound(if start {
                Sound::VoiceStart
            } else {
                Sound::VoiceStop
            });
        }
    }

    /// Registers the Voice object's waker and hands it the wakes QML has not read; a new one replaces the last, as the
    /// app's root may be made again.
    pub fn set_voice_waker(&self, waker: VoiceWaker) {
        let replaced = self.voice_waker().replace(Arc::new(waker)).is_some();
        if replaced {
            tracing::debug!("a new Voice object took the voice wakes");
        }
        for (unread, what) in [
            (&self.voice.view, VoiceWake::View),
            (&self.voice.meter, VoiceWake::Level),
        ] {
            if unread.waits() {
                self.voice_wake(what);
            }
        }
    }

    /// At the `ui` module's start: the module hears the rows, and a kept view is worded again.
    pub(super) fn voice_start(&self) {
        self.voice_rows();
        self.voice_reword();
    }

    /// The keyboard's language changed: a kept view is worded again.
    pub(super) fn voice_reword(&self) {
        if self.voice().page.is_some() {
            self.voice_woken(&self.voice.view, VoiceWake::View);
        }
    }

    /// A pad page tap turns the open panel's page; true when it did.
    pub(super) fn voice_pad(&self, back: bool) -> bool {
        let round = {
            let s = self.voice();
            match s.page.as_deref() {
                Some(VoicePage {
                    panel: Some(_),
                    round,
                    ..
                }) => *round,
                _ => return false,
            }
        };
        let click = VoiceClick::Page { back };
        self.voice_send(&VoiceAct { round, click });
        true
    }

    /// The place of the shown view's bar, Above before any view.
    pub(super) fn voice_place(&self) -> BarPlace {
        self.voice()
            .page
            .as_ref()
            .map_or(BarPlace::Above, |p| p.place)
    }

    /// Sends `event` through the limited handle, outside the lock; nothing goes before `listen`.
    pub(super) fn voice_send<E: Event>(&self, event: &E) {
        let handle = self.voice().bus.clone();
        bus::send(handle, event, "a voice event was not sent");
    }

    /// Keeps `page` and wakes; a bar that stops listening drops its level, and one that hides while the pointer is over
    /// it tells the module the pointer left.
    fn voice_view(&self, page: &Arc<VoicePage>) {
        let hidden = page.bar == VoiceBar::Hidden;
        let left = {
            let mut s = self.voice();
            s.page = Some(Arc::clone(page));
            let was = s.over;
            s.over = was && !hidden;
            was && hidden
        };
        if !matches!(page.bar, VoiceBar::Listening(_)) {
            self.voice.level.store(0, Ordering::SeqCst);
        }
        if left {
            let click = VoiceClick::Over(false);
            self.voice_send(&VoiceAct {
                round: page.round,
                click,
            });
        }
        self.voice_woken(&self.voice.view, VoiceWake::View);
    }

    /// Keeps `level` while the bar listens and wakes the meter; a level heard otherwise is late.
    fn voice_heard(&self, level: u8) {
        let listens = self
            .voice()
            .page
            .as_ref()
            .is_some_and(|p| matches!(p.bar, VoiceBar::Listening(_)));
        if listens {
            self.voice.level.store(level, Ordering::SeqCst);
            self.voice_woken(&self.voice.meter, VoiceWake::Level);
        }
    }

    /// Tells the module the rows a page holds, `[ui] page_rows`; it pages by the last one heard.
    fn voice_rows(&self) {
        let rows = self.lock().settings.page_rows;
        let rows = VoiceClick::Rows(u8::try_from(rows).unwrap_or(u8::MAX));
        let round = self.voice().page.as_ref().map_or(0, |p| p.round);
        self.voice_send(&VoiceAct { round, click: rows });
    }

    /// Compare's set text in the panel's language, the layout's in front, which the user reads aloud; `None` before a
    /// panel or for a language the UI has no text for. The panel's own words may be pinned to another language.
    fn set_text(&self) -> Option<String> {
        let iso = self.voice().page.as_ref()?.panel.as_ref()?.lang.iso.clone();
        let lang = spoken(&iso)?;
        let shown = self.i18n();
        let other;
        let t = if shown.lang().tag == lang.tag {
            &*shown
        } else {
            other = I18n::new(lang);
            &other
        };
        t.has(VOICE_READ_TEXT).then(|| t.text(VOICE_READ_TEXT))
    }

    /// Wakes the object with `what` unless a wake already waits for its read.
    fn voice_woken(&self, unread: &Unread, what: VoiceWake) {
        if unread.arm() {
            self.voice_wake(what);
        }
    }

    /// Calls the object's waker, if any, outside the lock.
    fn voice_wake(&self, what: VoiceWake) {
        let waker = self.voice_waker().clone();
        if let Some(waker) = waker {
            waker(what);
        }
    }

    fn voice(&self) -> MutexGuard<'_, State> {
        self.voice
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn voice_waker(&self) -> MutexGuard<'_, Option<Arc<VoiceWaker>>> {
        self.voice
            .waker
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

/// True when `page` shows Compare's set text.
fn reads(page: &VoicePage) -> bool {
    let sheet = page.panel.as_ref().map(|p| &p.sheet);
    matches!(sheet, Some(VoiceSheet::Compare(CompareSheet::Ready { .. })))
}
