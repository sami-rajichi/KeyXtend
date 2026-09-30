//! Sticky modifiers as in osk.exe: a click latches Shift, Ctrl, Alt, `AltGr` or Win. The `modifiers` setting
//! says whether the next key lets them go (`release`) or they stay until clicked again (`stay`).

use crate::settings::Modifiers;
use crate::spec::Latch;
use kx_module_api::ports::Scan;
use std::collections::BTreeMap;
use std::fmt;

/// The latched modifiers, each with the scan code of the key that latched it, and when they let go.
#[derive(Clone, PartialEq, Eq)]
pub struct Latches {
    on: BTreeMap<Latch, Scan>,
    mode: Modifiers,
}

/// Shows which modifiers are latched and the mode, never a key's scan code.
impl fmt::Debug for Latches {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Latches")
            .field("on", &self.on.keys().collect::<Vec<_>>())
            .field("mode", &self.mode)
            .finish()
    }
}

impl Latches {
    /// Nothing latched; `mode` says when latches let go.
    #[must_use]
    pub fn new(mode: Modifiers) -> Self {
        Self {
            on: BTreeMap::new(),
            mode,
        }
    }

    /// When the latches let go.
    #[must_use]
    pub fn mode(&self) -> Modifiers {
        self.mode
    }

    /// From the next key on, latches let go by `mode`; what is latched now stays.
    pub fn set_mode(&mut self, mode: Modifiers) {
        self.mode = mode;
    }

    /// A click on the modifier key sending `key`: latches `l`, or lets it go when latched.
    pub fn toggle(&mut self, l: Latch, key: Scan) {
        if self.on.remove(&l).is_none() {
            self.on.insert(l, key);
        }
    }

    /// True while `l` is latched.
    #[must_use]
    pub fn is_on(&self, l: Latch) -> bool {
        self.on.contains_key(&l)
    }

    /// The scan codes to hold around the next key, in a fixed order, leaving out `skip`.
    #[must_use]
    pub fn held(&self, skip: &[Latch]) -> Vec<Scan> {
        let kept = self.on.iter().filter(|(l, _)| !skip.contains(l));
        kept.map(|(_, &key)| key).collect()
    }

    /// A key went out: `release` lets every latch go, `stay` keeps them.
    pub fn used(&mut self) {
        if self.mode == Modifiers::Release {
            self.on.clear();
        }
    }
}

#[cfg(test)]
mod tests;
