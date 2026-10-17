//! The scroll pad as UI Automation reads it: the arrows stay in the tree, each describing its speed while it scrolls
//! and nothing while still, but Stop leaves the tree whenever the pad is idle, so idle is read from the arrows.
#![cfg(windows)]

use kx_module_api::ports::Dir;
use windows::Win32::Foundation::HWND;

use crate::keymap::Reader;
use crate::p5cfg::Pad;

/// True when none of the arrows' descriptions in `said` reads a speed.
pub fn idle_of(said: &[String]) -> bool {
    said.iter().all(String::is_empty)
}

/// Presses Stop with `press` unless `idle` reads the pad idle, and true when it pressed; a press that fails as the pad
/// stopped itself on the way counts as not needed.
pub fn halted(
    idle: impl Fn() -> Result<bool, String>,
    press: impl FnOnce() -> Result<(), String>,
) -> Result<bool, String> {
    if idle()? {
        return Ok(false);
    }
    let Err(e) = press() else {
        return Ok(true);
    };
    if idle()? { Ok(false) } else { Err(e) }
}

/// The pad of keyboard window `kb`, read by `reader` with `[pad]`'s ids.
pub struct PadRead<'a> {
    /// The UI Automation client.
    pub reader: &'a Reader,
    /// The keyboard window.
    pub kb: HWND,
    /// The pad's ids.
    pub pad: &'a Pad,
}

impl PadRead<'_> {
    /// What the arrow towards `dir` describes: its speed while it scrolls, else nothing.
    pub fn said(&self, dir: Dir) -> Result<String, String> {
        self.reader.described(self.kb, &self.pad.arrow(dir))
    }

    /// What Stop's dots describe; read it only while an arrow is lit, as Stop is gone from the tree otherwise.
    pub fn stop_said(&self) -> Result<String, String> {
        self.reader.described(self.kb, &self.pad.stop_id())
    }

    /// True when no arrow reads a speed.
    pub fn idle(&self) -> Result<bool, String> {
        let said: Result<Vec<String>, String> =
            Dir::ALL.into_iter().map(|d| self.said(d)).collect();
        Ok(idle_of(&said?))
    }

    /// Presses Stop by its id with `press` unless the pad is idle; true when it pressed.
    pub fn stop(&self, press: impl FnOnce(&str) -> Result<(), String>) -> Result<bool, String> {
        let id = self.pad.stop_id();
        halted(|| self.idle(), || press(&id))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn the_pad_is_idle_only_when_no_arrow_reads_a_speed() {
        let quiet = vec![String::new(); 4];
        assert!(idle_of(&quiet));
        let mut one = quiet.clone();
        one[2] = "Slow".into();
        assert!(!idle_of(&one));
    }

    #[test]
    fn stop_is_pressed_only_while_the_pad_scrolls() {
        let pressed = Cell::new(0);
        let press = || {
            pressed.set(pressed.get() + 1);
            Ok(())
        };
        assert_eq!(halted(|| Ok(true), press), Ok(false), "idle: no press");
        assert_eq!(pressed.get(), 0);
        assert_eq!(halted(|| Ok(false), press), Ok(true));
        assert_eq!(pressed.get(), 1);
    }

    #[test]
    fn a_stop_that_vanished_as_the_pad_stopped_itself_is_not_an_error() {
        let reads = Cell::new(0);
        let idle_after_one = || {
            reads.set(reads.get() + 1);
            Ok(reads.get() > 1)
        };
        let gone = || Err("no dpad.stop".to_string());
        assert_eq!(halted(idle_after_one, gone), Ok(false));
        assert_eq!(
            halted(|| Ok(false), gone),
            Err("no dpad.stop".to_string()),
            "still scrolling: a real failure"
        );
        let unread = halted(|| Err("no reader".to_string()), || Ok(()));
        assert_eq!(unread, Err("no reader".to_string()));
    }
}
