//! Sticky modifiers as in osk.exe: a click latches Shift, Ctrl, Alt, AltGr or Win, and the next key releases it.

use std::collections::BTreeMap;

use serde::Deserialize;

/// A modifier that latches; both Shift keys share one latch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Latch {
    /// Shift.
    Shift,
    /// Control.
    Ctrl,
    /// Left Alt.
    Alt,
    /// Right Alt, which is AltGr on layouts that have it.
    AltGr,
    /// The Windows key.
    Win,
}

/// The latched modifiers, each with the scan code of the key that latched it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Latches {
    on: BTreeMap<Latch, u32>,
}

impl Latches {
    /// A click on a modifier key sending `code`: latches it, or releases it when latched.
    pub fn toggle(&mut self, l: Latch, code: u32) {
        if self.on.remove(&l).is_none() {
            self.on.insert(l, code);
        }
    }

    /// True while `l` is latched.
    pub fn is_on(&self, l: Latch) -> bool {
        self.on.contains_key(&l)
    }

    /// The scan codes to hold around the next key, releasing every latch.
    pub fn take(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.on).into_values().collect()
    }

    /// Like `take`, but `skip` latches are only released, not held.
    pub fn take_but(&mut self, skip: &[Latch]) -> Vec<u32> {
        let on = std::mem::take(&mut self.on);
        on.into_iter()
            .filter(|(l, _)| !skip.contains(l))
            .map(|(_, c)| c)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_latches_and_a_second_click_releases() {
        let mut m = Latches::default();
        m.toggle(Latch::Shift, 0x2A);
        assert!(m.is_on(Latch::Shift));
        m.toggle(Latch::Shift, 0x36);
        assert!(!m.is_on(Latch::Shift), "either Shift key releases it");
    }

    #[test]
    fn the_next_key_takes_every_latch_once() {
        let mut m = Latches::default();
        m.toggle(Latch::Ctrl, 0x1D);
        m.toggle(Latch::Shift, 0x2A);
        assert_eq!(m.take(), [0x2A, 0x1D], "in a fixed order");
        assert!(m.take().is_empty(), "released after one key");
        assert!(!m.is_on(Latch::Ctrl));
    }

    #[test]
    fn a_skipped_latch_is_released_but_not_held() {
        let mut m = Latches::default();
        m.toggle(Latch::AltGr, 0xE038);
        m.toggle(Latch::Shift, 0x2A);
        assert_eq!(m.take_but(&[Latch::AltGr]), [0x2A]);
        assert!(!m.is_on(Latch::AltGr), "released too");
    }
}
