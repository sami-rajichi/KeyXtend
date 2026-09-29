//! Injection as a plain list of key events, built and tested on every system before Windows sees it.

use std::fmt;

use kx_module_api::REDACTED_MARKER;
use kx_module_api::ports::Scan;

use crate::keys;

/// What one key event presses or releases.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Key {
    /// A scan code alone; the target's layout decides what it types.
    Scan(Scan),
    /// A virtual key with its scan code, for shortcuts and the Keys route.
    Vk {
        /// The virtual key.
        vk: u16,
        /// Its scan code without the prefix byte.
        scan: u8,
        /// It goes as an extended key.
        extended: bool,
    },
    /// One UTF-16 unit of text, whatever the target's layout.
    Unit(u16),
}

impl Key {
    /// Virtual key `vk`, whose scan code in the layout is `mapped` (from `MapVirtualKeyExW`).
    #[must_use]
    pub fn vk(vk: u16, mapped: u32) -> Self {
        Self::Vk {
            vk,
            scan: mapped.to_le_bytes()[0],
            extended: keys::vk_extended(vk, mapped),
        }
    }

    /// True when the OS must get the extended-key flag.
    #[must_use]
    pub fn extended(self) -> bool {
        match self {
            Self::Scan(scan) => scan.is_extended(),
            Self::Vk { extended, .. } => extended,
            Self::Unit(_) => false,
        }
    }

    /// True for Alt or Win, whose lone release opens the menu bar or Start.
    fn opens_menu(self) -> bool {
        match self {
            Self::Scan(scan) => keys::ALT_WIN_SCANS.contains(&scan.code()),
            Self::Vk { vk, .. } => keys::ALT_WIN_VKS.contains(&vk),
            Self::Unit(_) => false,
        }
    }

    /// This key going down, or up when `up`.
    #[must_use]
    pub const fn event(self, up: bool) -> KeyEvent {
        KeyEvent { key: self, up }
    }
}

/// Shows only the kind of key, so a log never holds what was typed.
impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind = match self {
            Self::Scan(_) => "Scan",
            Self::Vk { .. } => "Vk",
            Self::Unit(_) => "Unit",
        };
        write!(f, "{kind}({REDACTED_MARKER})")
    }
}

/// One key going down, or up.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    /// The key.
    pub key: Key,
    /// True for the release.
    pub up: bool,
}

/// `keys` pressed in order, then released in reverse, such as Ctrl, Shift, T.
#[must_use]
pub fn combo(keys: &[Key]) -> Vec<KeyEvent> {
    let downs = keys.iter().map(|k| k.event(false));
    downs
        .chain(keys.iter().rev().map(|k| k.event(true)))
        .collect()
}

/// A press and a release for each UTF-16 unit of `text`, so a surrogate pair's halves stay side by side.
#[must_use]
pub fn unicode(text: &str) -> Vec<KeyEvent> {
    let units = text.encode_utf16().map(Key::Unit);
    units
        .flat_map(|k| [k.event(false), k.event(true)])
        .collect()
}

/// The releases, newest first, of the keys that `sent` pressed and did not release.
#[must_use]
pub fn still_down(sent: &[KeyEvent]) -> Vec<KeyEvent> {
    let mut down: Vec<Key> = Vec::new();
    for e in sent {
        if !e.up {
            down.push(e.key);
        } else if let Some(at) = down.iter().rposition(|&k| k == e.key) {
            down.remove(at);
        }
    }
    down.into_iter().rev().map(|k| k.event(true)).collect()
}

/// What lets go of the keys `sent` left down: their releases, after a tap of the mask key when
/// they let go of Alt or Win.
#[must_use]
pub fn releases(sent: &[KeyEvent]) -> Vec<KeyEvent> {
    let ups = still_down(sent);
    if !ups.iter().any(|e| e.key.opens_menu()) {
        return ups;
    }
    let mask = Key::vk(keys::MASK_VK, 0);
    [mask.event(false), mask.event(true)]
        .into_iter()
        .chain(ups)
        .collect()
}

#[cfg(test)]
mod tests;
