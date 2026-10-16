//! A mode or latch key of the shown keyboard: read on or off through UI Automation, turned with a click, and waited
//! for until it reads so.
#![cfg(windows)]

use kx_mod_keyboard::Role;

use crate::keymap::Reader;
use crate::product::Keyboard;
use crate::{board, mouse, win};

/// How long to wait for a key to read on or off after its click, and how often to look, in ms.
#[derive(Debug, Clone, Copy)]
pub struct Wait {
    /// The longest wait.
    pub lit_ms: u64,
    /// How often it looks.
    pub poll_ms: u64,
}

/// A key with an on or off state on the keyboard shown when it was found.
pub struct ModeKey {
    id: String,
    reader: Reader,
    /// The keyboard it sits on.
    pub keyboard: Keyboard,
}

impl ModeKey {
    /// The key with `role` on the keyboard shown now.
    pub fn of(role: Role) -> Result<Self, String> {
        Self::by_id(board::id_of(role)?)
    }

    /// The key with board id `id` on the keyboard shown now.
    pub fn by_id(id: String) -> Result<Self, String> {
        Ok(Self {
            id,
            reader: Reader::new()?,
            keyboard: Keyboard::find()?,
        })
    }

    /// Whether it reads on now.
    pub fn on(&self) -> Result<bool, String> {
        let read = self.reader.toggled(self.keyboard.hwnd, &self.id)?;
        read.ok_or_else(|| format!("key {} has no on or off state", self.id))
    }

    /// Turns it `on` or off with a click, unless it reads so already, and waits for it to read so; refused when another
    /// window covers it.
    pub fn set(&self, on: bool, wait: Wait) -> Result<(), String> {
        if self.on()? == on {
            return Ok(());
        }
        mouse::click_at(self.keyboard.top_at(&self.id)?)?;
        self.wait(on, wait)
    }

    /// As `set`, for a step that puts things back: a refused click, as after the mouse guard's stop, becomes a UI
    /// Automation press, which still goes out.
    pub fn put_back(&self, on: bool, wait: Wait) -> Result<(), String> {
        if self.on()? == on {
            return Ok(());
        }
        let clicked = self.keyboard.top_at(&self.id).and_then(mouse::click_at);
        or_press(clicked, || {
            self.reader.restore(self.keyboard.hwnd, &self.id)
        })?;
        self.wait(on, wait)
    }

    /// Waits up to `wait.lit_ms` for it to read `on`, as after the product changed it itself.
    pub fn wait(&self, on: bool, wait: Wait) -> Result<(), String> {
        let lit = || self.on().ok().filter(|now| *now == on).map(drop);
        let word = if on { "on" } else { "off" };
        win::poll_until(wait.lit_ms, wait.poll_ms, lit).ok_or_else(|| {
            let id = &self.id;
            format!("key {id} did not read {word} within {} ms", wait.lit_ms)
        })
    }
}

/// `clicked`, unless the click was refused: then `press`, which still goes out after the mouse guard's stop.
fn or_press(
    clicked: Result<(), String>,
    press: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    clicked.or_else(|_| press())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn a_refused_click_becomes_a_press_and_a_click_that_went_out_needs_none() {
        let pressed = Cell::new(0);
        let press = || {
            pressed.set(pressed.get() + 1);
            Ok(())
        };
        assert_eq!(or_press(Ok(()), press), Ok(()));
        assert_eq!(pressed.get(), 0, "the click went out");
        let refused = Err("stopped by the mouse guard".to_string());
        assert_eq!(or_press(refused, press), Ok(()));
        assert_eq!(pressed.get(), 1);
        let gone = or_press(Err("covered".into()), || Err("no such button".into()));
        assert_eq!(
            gone,
            Err("no such button".to_string()),
            "the press's own error"
        );
    }
}
