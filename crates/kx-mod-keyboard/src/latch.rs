//! Sticky modifiers as in osk.exe: a click latches Shift, Ctrl, Alt, `AltGr` or Win. The `modifiers` setting
//! says whether the next key lets them go (`release`) or they stay until clicked again (`stay`).

use crate::settings::Modifiers;
use crate::spec::Latch;
use kx_module_api::ports::Scan;
use std::collections::BTreeMap;
use std::fmt;

/// One latch that is on: the scan code of the key that latched it, and whether anything used it since.
#[derive(Copy, Clone, PartialEq, Eq)]
struct On {
    key: Scan,
    used: bool,
}

/// The latched modifiers, each with the scan code of the key that latched it, and when they let go.
#[derive(Clone, PartialEq, Eq)]
pub struct Latches {
    on: BTreeMap<Latch, On>,
    mode: Modifiers,
}

/// Shows only the mode: which modifiers are latched stays hidden, as a run of clicks would show what was typed.
impl fmt::Debug for Latches {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Latches")
            .field("mode", &self.mode)
            .finish_non_exhaustive()
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

    /// A click on the modifier key sending `key`: latches `l`, which uses every latch already on, or lets it go.
    pub fn toggle(&mut self, l: Latch, key: Scan) {
        if self.on.remove(&l).is_none() {
            self.spend();
            self.on.insert(l, On { key, used: false });
        }
    }

    /// True while `l` is latched.
    #[must_use]
    pub fn is_on(&self, l: Latch) -> bool {
        self.on.contains_key(&l)
    }

    /// True while `l` is latched and no key, click or later latch has used it.
    #[must_use]
    pub fn unused(&self, l: Latch) -> bool {
        self.on.get(&l).is_some_and(|on| !on.used)
    }

    /// The scan codes to hold around the next key, in a fixed order, leaving out `skip`.
    #[must_use]
    pub fn held(&self, skip: &[Latch]) -> Vec<Scan> {
        let kept = self.on.iter().filter(|(l, _)| !skip.contains(l));
        kept.map(|(_, on)| on.key).collect()
    }

    /// A key or a click went out: `release` lets every latch go, `stay` keeps them, used. True when any let go.
    pub fn used(&mut self) -> bool {
        self.spend();
        let go = self.mode == Modifiers::Release && !self.on.is_empty();
        if go {
            self.on.clear();
        }
        go
    }

    /// Lets every latch go whatever the mode, as before a paste the clipboard module asked for. True when any let go.
    pub fn clear(&mut self) -> bool {
        let any = !self.on.is_empty();
        self.on.clear();
        any
    }

    /// Something went out that keeps the latches, such as Caps Lock: each stays on, used.
    pub fn spend(&mut self) {
        for on in self.on.values_mut() {
            on.used = true;
        }
    }

    /// As `used`, for a word that started under `then`: only the latches `then` held and still on count, so one
    /// clicked during the word stays. True when any let go.
    pub fn used_of(&mut self, then: &Self) -> bool {
        self.spend_of(then);
        let held: Vec<Latch> = self.on.keys().copied().filter(|l| then.is_on(*l)).collect();
        let go = self.mode == Modifiers::Release && !held.is_empty();
        if go {
            self.on.retain(|l, _| !held.contains(l));
        }
        go
    }

    /// As `spend`, for a word that started under `then`: only the latches `then` held are used.
    pub fn spend_of(&mut self, then: &Self) {
        for (l, on) in &mut self.on {
            on.used |= then.is_on(*l);
        }
    }
}

#[cfg(test)]
mod tests;
