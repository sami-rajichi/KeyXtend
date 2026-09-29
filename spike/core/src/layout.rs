//! Key labels and rows for the spike faces, read through the product's layout source (ADR-0014).
//! `Follow` moves to kx-mod-layouts and the rows to kx-mod-keyboard later in P3.

use kx_module_api::ports::{LayoutSource, Scan};
use kx_platform_windows::{WinLayouts, front, layouts};
use serde::Serialize;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyNameTextW, HKL};

use crate::config::KeyboardConfig;
use crate::place::{Place, places};

/// Bit position of the scan code in a `GetKeyNameTextW` argument.
const NAME_SCAN_SHIFT: u32 = 16;
/// Extended-key bit in a `GetKeyNameTextW` argument.
const NAME_EXTENDED_BIT: i32 = 1 << 24;
/// Longest key name read from Windows.
const LABEL_CAP: usize = 32;
/// Width of a key that `widths` in spike.toml does not list, in key units.
pub const DEFAULT_WIDTH: f32 = 1.0;

/// One key: its scan code, label and box.
#[derive(Debug, Clone, Serialize)]
pub struct KeyCap {
    /// Scan code; `0xE0xx` marks an extended key.
    pub code: u32,
    /// Text shown on the key.
    pub label: String,
    /// Where the key sits; JSON gets its fields as `x`, `y`, `w`, `h`.
    #[serde(flatten)]
    pub place: Place,
}

/// Spike scan code `code` as the product's `Scan`.
pub fn scan(code: u32) -> Scan {
    Scan::new(code as u16)
}

/// The label for `code` in `hkl`: its character, or its key name for non-printing keys.
pub fn label(code: u32, hkl: HKL) -> String {
    let printed = character(code, hkl);
    if layouts::is_printable(&printed) {
        printed
    } else {
        key_name(code)
    }
}

/// What `code` types in `hkl` with no modifiers; empty for non-printing keys.
pub fn character(code: u32, hkl: HKL) -> String {
    let keys = WinLayouts.keys(layouts::id(hkl), &[scan(code)]);
    keys.into_iter().next().map(|k| k.base).unwrap_or_default()
}

fn key_name(code: u32) -> String {
    let key = scan(code);
    let mut arg = (u32::from(key.byte()) << NAME_SCAN_SHIFT) as i32;
    if key.is_extended() {
        arg |= NAME_EXTENDED_BIT;
    }
    let mut buf = [0u16; LABEL_CAP];
    // SAFETY: the buffer is valid for its whole length.
    let n = unsafe { GetKeyNameTextW(arg, &mut buf) };
    String::from_utf16_lossy(&buf[..usize::try_from(n).unwrap_or(0)])
}

/// The configured rows with labels from `hkl` and each key's box.
pub fn rows(cfg: &KeyboardConfig, hkl: HKL) -> Vec<Vec<KeyCap>> {
    cfg.rows
        .iter()
        .zip(places(cfg))
        .map(|(row, boxes)| {
            row.iter()
                .zip(boxes)
                .map(|(&code, place)| KeyCap {
                    code,
                    label: label(code, hkl),
                    place,
                })
                .collect()
        })
        .collect()
}

/// Remembers which layout the labels show, to relabel only when the app in front changes it.
#[derive(Debug, Default)]
pub struct Follow {
    shown: isize,
}

impl Follow {
    /// Starts from the labels of `hkl`.
    pub fn new(hkl: HKL) -> Self {
        Self {
            shown: hkl.0 as isize,
        }
    }

    /// The new layout when another app is in front and its layout differs; else `None`.
    pub fn changed(&mut self) -> Option<HKL> {
        let front = crate::window::foreground();
        let now = front::layout();
        let other = !front.is_invalid() && !crate::window::is_ours(front);
        self.step(other, now.0 as isize).then_some(now)
    }

    fn step(&mut self, other_app_in_front: bool, now: isize) -> bool {
        let new = other_app_in_front && now != self.shown;
        if new {
            self.shown = now;
        }
        new
    }
}

/// The configured rows as JSON, for faces that build keys in QML or HTML.
pub fn rows_json(cfg: &KeyboardConfig, hkl: HKL) -> String {
    serde_json::to_string(&rows(cfg, hkl)).unwrap_or_default()
}

/// Width of `code` in key units, from `widths` in spike.toml.
pub fn width(cfg: &KeyboardConfig, code: u32) -> f32 {
    cfg.widths
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(DEFAULT_WIDTH, |(_, w)| *w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_read_through_the_product_and_other_keys_by_name() {
        // Scan codes: Space and Escape have names, not characters.
        let hkl = front::layout();
        for code in [0x39, 0x01] {
            assert_eq!(character(code, hkl), "", "{code:#x}");
            assert!(!label(code, hkl).is_empty(), "{code:#x}");
        }
        assert_eq!(scan(0xE035), Scan::new(0xE035));
    }

    #[test]
    fn follow_relabels_only_for_another_app_with_a_new_layout() {
        let mut f = Follow { shown: 1 };
        assert!(!f.step(true, 1));
        assert!(!f.step(false, 2), "our own window or none in front");
        assert!(f.step(true, 2));
        assert!(!f.step(true, 2), "already showing it");
    }

    #[test]
    fn width_uses_the_table_else_the_default() {
        let cfg = KeyboardConfig {
            title: String::new(),
            key_px: 48.0,
            gap_px: 4.0,
            font_px: 16.0,
            guard_delay_ms: 0,
            guard_tries: 1,
            relabel_ms: 0,
            rows: vec![],
            widths: vec![(0x0F, 1.5)],
        };
        assert_eq!(width(&cfg, 0x0F), 1.5);
        assert_eq!(width(&cfg, 0x10), DEFAULT_WIDTH);
    }
}
