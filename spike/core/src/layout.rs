//! Key labels and rows for the spike faces, read through the product's layout source (ADR-0014).
//! `Follow` moved to kx-mod-layouts (P3 Task 10); the rows go to kx-mod-keyboard later in P3.

use kx_module_api::ports::{LayoutId, LayoutSource, Scan};
use kx_platform_windows::{WinLayouts, layouts, window};
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

/// The layout of the app in front, else the first installed one, read through `WinLayouts::foreground`.
pub fn front_layout() -> HKL {
    let id = WinLayouts
        .foreground()
        .or_else(|| WinLayouts.installed().first().copied());
    id.and_then(layouts::hkl).unwrap_or_default()
}

/// The layout of the app in front when it is another program's; `None` for our own window or none,
/// so kx-mod-layouts' `Follow` keeps the shown layout.
pub fn other_front() -> Option<LayoutId> {
    let front = window::foreground();
    let other = !front.is_invalid() && !window::is_ours(front);
    WinLayouts.foreground().filter(|_| other)
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
        let hkl = front_layout();
        for code in [0x39, 0x01] {
            assert_eq!(character(code, hkl), "", "{code:#x}");
            assert!(!label(code, hkl).is_empty(), "{code:#x}");
        }
        assert_eq!(scan(0xE035), Scan::new(0xE035));
    }

    #[test]
    fn the_front_layout_is_an_installed_one() {
        let front = layouts::id(front_layout());
        assert!(WinLayouts.installed().contains(&front), "{front}");
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
