//! The selection strip on the hub (ADR-0041): it keeps the last `ChipsView` and wakes QML once per read, places the strip
//! QML measured by its line and clear of our windows, sends its clicks as `ChipAct` through a handle limited to the `ui`
//! module's capabilities, and tells the keyboard's place at each change and on ask. A ticked strip stays until QML's
//! fade ends; nothing here logs a box.

mod place;
mod view;

use super::unread::Unread;
use super::{Hub, NULL, Wake, bus, json};
use crate::i18n::keys::sel_chip_id;
use crate::ui_module::MANIFEST;
use kx_module_api::ports::{
    BoardPlace, BoardPlaceAsk, ChipAct, ChipsView, Legend, Point, Rect, SelChip, StripSpot,
    physical,
};
use kx_module_api::{Bus, Event, EventError, Subscription};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The strip's side of the hub: what it shows, and whether a wake waits for QML's read.
#[derive(Default)]
pub(super) struct Sel {
    state: Mutex<State>,
    unread: Unread,
}

/// The strip shown, and the handle that sends its clicks and the keyboard's place.
#[derive(Default)]
struct State {
    /// The bus limited to the `ui` module's capabilities, once `listen` has it.
    bus: Option<Bus>,
    /// The strip shown, ticked or not; none while hidden.
    strip: Option<Kept>,
}

/// A strip as the module showed it, its screen known and its boxes sane.
#[derive(Clone, Debug)]
struct Kept {
    round: u32,
    chips: Vec<SelChip>,
    spot: StripSpot,
    acted: Option<SelChip>,
    legend: Legend,
}

impl Kept {
    /// The strip `view` shows; none for `Hidden`, no chips, an unknown screen or boxes that make no sense.
    fn of(view: ChipsView) -> Option<Self> {
        let ChipsView::Shown {
            round,
            chips,
            spot,
            acted,
            legend,
        } = view
        else {
            return None;
        };
        let shows = !chips.is_empty() && place::sane(&spot);
        shows.then_some(Self {
            round,
            chips,
            spot,
            acted,
            legend,
        })
    }

    /// True when this strip is the one of round `round`, as QML names it.
    fn is(&self, round: i64) -> bool {
        i64::from(self.round) == round
    }
}

impl Hub {
    /// Keeps a handle limited to the `ui` module's capabilities for the clicks and the place, then hears the strip's
    /// views and the place's ask through it, so an ask heard at once is answered.
    pub(super) fn hear_sel(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        let gated = bus.restricted(MANIFEST.capabilities);
        gated.check::<ChipAct>()?;
        gated.check::<BoardPlace>()?;
        self.sel().bus = Some(gated.clone());
        Ok(vec![
            self.hear(&gated, |hub, v: &ChipsView| hub.sel_view(v.clone()))?,
            self.hear(&gated, |hub, _: &BoardPlaceAsk| hub.tell_board())?,
        ])
    }

    /// The strip as JSON for QML, worded in the keyboard's language, or `null` while none shows; reading it lets the
    /// next change wake again.
    #[must_use]
    pub fn sel_json(&self) -> String {
        self.sel.unread.read();
        let kept = self.sel().strip.clone();
        kept.map_or_else(|| NULL.to_owned(), |k| json(&view::of(&k, &self.i18n())))
    }

    /// The top-left, in physical pixels, of the strip of round `round` that QML measured `w` by `h` physical pixels for
    /// a keyboard at `size`: the theme's `[chips]` gap for that size from its line, clear of `clear` (our windows'
    /// boxes) when there is room, on its work area. `None` for another round, no strip, a size that is not above 0 or
    /// a strip that does not fit that work area.
    #[must_use]
    pub fn sel_at(&self, round: i64, w: i32, h: i32, size: f64, clear: &[Rect]) -> Option<Point> {
        let spot = self.sel().strip.as_ref().filter(|k| k.is(round))?.spot;
        let gap = (size > 0.0).then(|| self.themes.shape.chips.gap(size))?;
        let gap = physical(gap, spot.screen.dpi);
        place::at(&spot, w, h, gap, clear)
    }

    /// A click on keycap `chip`, an id from `sel_json`, of the strip of round `round`, on the Qt thread: it sounds as a
    /// key does and sends `ChipAct` while that strip shows the keycap with no tick. Any other click goes quietly, and an
    /// unknown id warns.
    pub fn sel_act(&self, round: i64, chip: &str) {
        self.tapped();
        let Some(chip) = SelChip::ALL.into_iter().find(|&c| sel_chip_id(c) == chip) else {
            tracing::warn!("QML sent an unknown selection keycap, so nothing was sent");
            return;
        };
        let live = self
            .sel()
            .strip
            .as_ref()
            .is_some_and(|k| k.is(round) && k.acted.is_none() && k.chips.contains(&chip));
        match u32::try_from(round) {
            Ok(round) if live => self.send_sel(&ChipAct { round, chip }),
            _ => tracing::debug!("a late selection click was dropped"),
        }
    }

    /// QML's fade of the ticked strip of round `round` ended, so the hub forgets it; any other strip stays.
    pub fn sel_faded(&self, round: i64) {
        let gone = self
            .sel()
            .strip
            .take_if(|k| k.is(round) && k.acted.is_some());
        drop(gone);
    }

    /// Tells the keyboard's place, and again while the mode changed meanwhile, so the last place told is the mode now.
    pub(super) fn tell_board(&self) {
        let mut told = self.mode();
        loop {
            self.send_sel(&BoardPlace {
                place: told.board(),
            });
            let now = self.mode();
            if now == told {
                return;
            }
            told = now;
        }
    }

    /// The keyboard's language changed: a kept strip is worded again.
    pub(super) fn sel_reword(&self) {
        if self.sel().strip.is_some() {
            self.sel_woken();
        }
    }

    /// Hands a new waker the strip's wake still waiting for a read.
    pub(super) fn rewake_sel(&self) {
        if self.sel.unread.waits() {
            self.wake(Wake::Sel);
        }
    }

    /// Keeps the strip `view` shows, or none, and wakes unless nothing showed before or after.
    fn sel_view(&self, view: ChipsView) {
        let kept = Kept::of(view);
        let shows = kept.is_some();
        let last = std::mem::replace(&mut self.sel().strip, kept);
        if shows || last.is_some() {
            self.sel_woken();
        }
    }

    /// Wakes QML unless a wake already waits for its read.
    fn sel_woken(&self) {
        if self.sel.unread.arm() {
            self.wake(Wake::Sel);
        }
    }

    /// Sends `event` through the limited handle, outside the lock; nothing goes before `listen`.
    fn send_sel<E: Event>(&self, event: &E) {
        let handle = self.sel().bus.clone();
        bus::send(handle, event, "a selection strip event was not sent");
    }

    fn sel(&self) -> MutexGuard<'_, State> {
        self.sel
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
