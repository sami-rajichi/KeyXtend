//! Window and status-line sizes: the key block from spike-core plus one status line.

use serde::Serialize;
use spike_core::config::KeyboardConfig;
use spike_core::place::block_size;

/// Sizes the page and the window share, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Sizes {
    /// Label and status font size.
    pub font: f64,
    /// Space left of the status text.
    pub gap: f64,
    /// Top of the status line, which is the bottom of the key block.
    pub top: f64,
    /// Height of the status line.
    pub status: f64,
    /// Window client width.
    pub width: f64,
    /// Window client height.
    pub height: f64,
}

impl Sizes {
    /// The key block, then a status line with one gap above and one below its text.
    pub fn new(cfg: &KeyboardConfig) -> Self {
        let (width, top) = block_size(cfg);
        let font = f64::from(cfg.font_px);
        let gap = f64::from(cfg.gap_px);
        let status = font + 2.0 * gap;
        Self {
            font,
            gap,
            top: f64::from(top),
            status,
            width: f64::from(width),
            height: f64::from(top) + status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_is_the_key_block_plus_the_status_line() {
        let cfg = KeyboardConfig {
            titles: Default::default(),
            key_px: 48.0,
            gap_px: 4.0,
            font_px: 16.0,
            guard_delay_ms: 0,
            guard_tries: 1,
            relabel_ms: 0,
            rows: vec![vec![0x29, 0x02], vec![0x39]],
            widths: vec![(0x39, 2.0)],
        };
        let want = Sizes {
            font: 16.0,
            gap: 4.0,
            top: 108.0,
            status: 24.0,
            width: 108.0,
            height: 132.0,
        };
        assert_eq!(Sizes::new(&cfg), want);
    }
}
