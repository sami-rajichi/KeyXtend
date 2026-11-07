//! The chip lane on the hub (ADR-0033): it keeps the last `Suggested` and wakes the top bar, and sends chip and menu
//! taps and the password box's Quick-fill chip through a handle limited to the `ui` module's capabilities, so the
//! bus's gate holds for the UI too.

mod view;

use super::{Hub, Wake, bus, json};
use crate::ui_module::MANIFEST;
use kx_module_api::ports::{ChipPicked, FillOpen, Suggested, WordsAsk};
use kx_module_api::{Bus, Event, EventError, Subscription};
use std::sync::{Arc, MutexGuard, PoisonError};

/// The kinds of tap QML names to `Hub::words_ask`.
pub mod asks {
    /// Forget the next chip tapped.
    pub const FORGET: &str = "forget";
    /// End forget mode, or close the email list.
    pub const CANCEL: &str = "cancel";
    /// Pause learning at 1, resume it at 0.
    pub const PAUSE: &str = "pause";
    /// Turn prediction on at 1, off at 0.
    pub const POWER: &str = "power";
    /// Bring back the word just forgotten.
    pub const UNDO: &str = "undo";
    /// Open the email list, as the More chip does.
    pub const MORE: &str = "more";
    /// Turn the email list by the argument's pages, back when below 0.
    pub const PAGE: &str = "page";
    /// Pick the email list's row at the argument's index.
    pub const PICK: &str = "pick";
    /// Send the lane again.
    pub const AGAIN: &str = "again";
    /// Open or close Quick-fill, as the password box's chip does.
    pub const FILL: &str = "fill";
}

/// The lane the top bar draws, and the handle that sends its taps.
#[derive(Default)]
pub(super) struct Words {
    /// The bus limited to the `ui` module's capabilities, once `listen` has it.
    bus: Option<Bus>,
    /// The last lane; none before the first.
    last: Option<Suggested>,
    /// The vault runs, as its last `FillLive` said, so the password box shows its Quick-fill chip.
    fill: bool,
}

impl Hub {
    /// Hears the chip lane through a handle limited to the `ui` module's capabilities, keeps that handle for the taps,
    /// and asks for the lane again.
    pub(super) fn hear_words(self: &Arc<Self>, bus: &Bus) -> Result<Subscription, EventError> {
        let gated = bus.restricted(MANIFEST.capabilities);
        let heard = self.hear(&gated, |hub, lane: &Suggested| hub.suggested(lane.clone()))?;
        gated.check::<ChipPicked>()?;
        gated.publish(&WordsAsk::Again)?;
        self.words().bus = Some(gated);
        Ok(heard)
    }

    /// The chip lane as JSON for the top bar: its round and kind, the chips with the names readers hear, the menu's
    /// switches, the email list's page while it is open and whether the Quick-fill chip shows.
    #[must_use]
    pub fn words_json(&self) -> String {
        let (last, fill) = {
            let words = self.words();
            (words.last.clone(), words.fill)
        };
        json(&view::of(last.as_ref(), fill, &self.i18n()))
    }

    /// Sends a tap on chip `index` of lane `round`, the round QML drew, so the module drops a tap on an old lane, and
    /// sounds as a key does; a round or index out of range sends nothing. Call it on the Qt thread.
    pub fn pick_chip(&self, round: i64, index: i32) {
        if let (Ok(round), Ok(index)) = (u32::try_from(round), u8::try_from(index)) {
            self.send(&ChipPicked { round, index });
        } else {
            tracing::warn!("QML sent a chip tap out of range, so nothing was sent");
        }
        self.tapped();
    }

    /// Sends the ask `kind` names (one of `asks`) with `arg` and a list's rows from `[ui] page_rows`, `fill` as
    /// `FillOpen` only while the vault runs, and sounds as a key does unless it is the lane's own `again`; an unknown
    /// kind or an `arg` out of range sends nothing. Call it on the Qt thread.
    pub fn words_ask(&self, kind: &str, arg: i32) {
        let rows = self.lock().settings.page_rows;
        let rows = u8::try_from(rows).unwrap_or(u8::MAX);
        if kind == asks::FILL {
            let live = self.words().fill;
            if live {
                self.send(&FillOpen);
            }
        } else if let Some(ask) = ask_of(kind, arg, rows) {
            self.send(&ask);
        } else {
            tracing::warn!(kind, arg, "QML sent an unknown words tap or a bad argument");
        }
        if kind != asks::AGAIN {
            self.tapped();
        }
    }

    /// Keeps whether the vault runs, and a change wakes the top bar, so the Quick-fill chip comes and goes at once.
    pub(super) fn fill_live(&self, live: bool) {
        let was = std::mem::replace(&mut self.words().fill, live);
        if was != live {
            self.wake(Wake::Words);
        }
    }

    /// Keeps `lane` and wakes the top bar.
    fn suggested(&self, lane: Suggested) {
        self.words().last = Some(lane);
        self.wake(Wake::Words);
    }

    /// Sends `event` through the limited handle, outside the lock; nothing goes before `listen`.
    fn send<E: Event>(&self, event: &E) {
        let handle = self.words().bus.clone();
        bus::send(handle, event, "a words tap was not sent");
    }

    fn words(&self) -> MutexGuard<'_, Words> {
        self.words.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The ask `kind` names with `arg`, `rows` to a list page; none for an unknown kind or an `arg` out of range.
fn ask_of(kind: &str, arg: i32, rows: u8) -> Option<WordsAsk> {
    let flag = || match arg {
        0 => Some(false),
        1 => Some(true),
        _ => None,
    };
    Some(match kind {
        asks::FORGET => WordsAsk::Forget,
        asks::CANCEL => WordsAsk::Cancel,
        asks::PAUSE => WordsAsk::Pause(flag()?),
        asks::POWER => WordsAsk::Power(flag()?),
        asks::UNDO => WordsAsk::Undo,
        asks::MORE => WordsAsk::More { rows },
        asks::PAGE => WordsAsk::Page {
            page: i8::try_from(arg).ok()?,
            rows,
        },
        asks::PICK => WordsAsk::ListPick(u8::try_from(arg).ok()?),
        asks::AGAIN => WordsAsk::Again,
        _ => return None,
    })
}

#[cfg(test)]
pub(super) mod tests;
