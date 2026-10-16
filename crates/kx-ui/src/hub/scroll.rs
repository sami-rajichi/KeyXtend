//! The scroll pad's side of the hub: the scroll state and the box to outline as one JSON, our open list's page turns,
//! and the guard's two scroll calls. A bus call only stores and wakes, at most one `Wake::Scroll` waits for QML's read,
//! and nothing here logs a box.

use super::face::done;
use super::unread::Unread;
use super::{Hub, Wake, json};
use kx_module_api::ports::{Dir, Rect, ScrollChanged};
use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The scroll state as QML reads it.
#[derive(Copy, Clone, PartialEq, Eq, Serialize)]
struct Shown {
    /// The scroll module runs.
    live: bool,
    /// The way it scrolls, null while still.
    dir: Option<Dir>,
    /// It scrolls at the fast speed.
    fast: bool,
    /// The box to outline in physical pixels, null for none; only a live module has one.
    area: Option<Rect>,
}

impl Default for Shown {
    fn default() -> Self {
        Self::from(ScrollChanged::OFF)
    }
}

/// A module that is not live is still and slow, whatever else its state says.
impl From<ScrollChanged> for Shown {
    fn from(now: ScrollChanged) -> Self {
        let ScrollChanged { live, dir, fast } = if now.live { now } else { ScrollChanged::OFF };
        Self {
            live,
            dir,
            fast,
            area: None,
        }
    }
}

/// The scroll state, and whether a wake waits for QML to read it.
#[derive(Default)]
pub(super) struct Scroll {
    shown: Mutex<Shown>,
    unread: Unread,
}

impl Scroll {
    fn lock(&self) -> MutexGuard<'_, Shown> {
        self.shown.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The open list as the keyboard's state JSON holds it: its rows, the highlighted one and the page buttons' keys.
#[derive(Deserialize)]
struct List {
    rows: Vec<IgnoredAny>,
    hi: usize,
    page_back: Option<String>,
    page_on: Option<String>,
}

/// The part of the keyboard's state the page turn reads.
#[derive(Deserialize)]
struct State {
    #[serde(default)]
    list: Option<List>,
}

impl List {
    /// The key of the page button that turns back or on, as the list's own button sends it; none at that end.
    /// The same page maths lives in `qml/LangList.qml` (`pages`, `at`) and kx-mod-keyboard's `LangList::key`, so change
    /// all three together.
    fn button(self, back: bool, rows: usize) -> Option<String> {
        let per = rows.max(1);
        let pages = self.rows.len().div_ceil(per).max(1);
        let at = (self.hi / per).min(pages - 1);
        match back {
            true if at > 0 => self.page_back,
            false if at + 1 < pages => self.page_on,
            _ => None,
        }
    }
}

impl Hub {
    /// The scroll state and the box to outline as JSON for QML, which lets the next change wake again; it holds a box,
    /// so never log it.
    #[must_use]
    pub fn scroll_json(&self) -> String {
        self.scroll.unread.read();
        json(&*self.scroll.lock())
    }

    /// Takes the scroll module's state for the outline and the badge; a module that stops takes its outline along.
    pub(super) fn scroll_changed(&self, now: ScrollChanged) {
        self.scroll_change(|s| {
            let area = s.area.filter(|_| now.live);
            *s = Shown {
                area,
                ..Shown::from(now)
            };
        });
        self.badge_scrolled(now);
    }

    /// Takes the box to outline, or none; only a live module outlines.
    pub(super) fn scroll_aimed(&self, area: Option<Rect>) {
        self.scroll_change(|s| {
            if s.live {
                s.area = area;
            }
        });
    }

    /// Hands a new waker the scroll wake still waiting for a read, as the last waker's QML may never have read it.
    pub(super) fn rewake_scroll(&self) {
        if self.scroll.unread.waits() {
            self.wake(Wake::Scroll);
        }
    }

    /// Applies `f`; a change wakes QML unless a wake already waits for its read.
    fn scroll_change(&self, f: impl FnOnce(&mut Shown)) {
        let changed = {
            let mut s = self.scroll.lock();
            let before = *s;
            f(&mut s);
            *s != before
        };
        if changed && self.scroll.unread.arm() {
            self.wake(Wake::Scroll);
        }
    }

    /// Turns our open list a page back or on, as its own page button does with `[ui] page_rows`; nothing at either
    /// end or with no list open. A turned page wakes the keyboard to read its state; call it on the Qt thread.
    pub fn turn_page(&self, back: bool) {
        let Some(face) = self.face() else {
            return;
        };
        let state = match serde_json::from_str::<State>(&face.state_json()) {
            Ok(state) => state,
            Err(e) => {
                // The error's text may quote the state, so only its kind is logged.
                tracing::warn!(kind = ?e.classify(), "the keyboard's state does not read, so no page turns");
                return;
            }
        };
        let rows = self.lock().settings.page_rows;
        let Some(key) = state.list.and_then(|l| l.button(back, rows)) else {
            return;
        };
        if self.panel_key(&key).state {
            self.wake(Wake::Keys);
        }
    }

    /// From now on keeps the last point the pointer had outside our window `title` each time it enters it, for the
    /// scroll target; false while it does not show or when refused. Call it on the UI thread.
    #[must_use]
    pub fn watch_entries(&self, title: &str) -> bool {
        let result = self.ports.guard.watch_entries(title);
        done(result, "the keyboard's entries will not be watched")
    }

    /// Puts our windows titled `title` just under our window `under`, as the outline goes under the keyboard; false
    /// while either does not show or when refused. Call it on the UI thread.
    #[must_use]
    pub fn lay_under(&self, title: &str, under: &str) -> bool {
        let result = self.ports.guard.lay_under(title, under);
        done(result, "a window could not be laid under another")
    }
}
