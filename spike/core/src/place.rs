//! Where each key sits, in logical pixels: one calculation shared by every face and the harness.

use serde::Serialize;

use crate::config::KeyboardConfig;
use crate::layout::width;

/// One key's box, from the top-left corner of the window's client area.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Place {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Every key's box, row by row: the block starts at (gap, gap); a key `u` units wide takes `u` pitches.
pub fn places(cfg: &KeyboardConfig) -> Vec<Vec<Place>> {
    let (key, gap) = (cfg.key_px, cfg.gap_px);
    let pitch = key + gap;
    cfg.rows
        .iter()
        .enumerate()
        .map(|(r, row)| {
            let mut x = gap;
            row.iter()
                .map(|&code| {
                    let u = width(cfg, code);
                    let place = Place {
                        x,
                        y: gap + r as f32 * pitch,
                        w: u * key + (u - 1.0) * gap,
                        h: key,
                    };
                    x += u * pitch;
                    place
                })
                .collect()
        })
        .collect()
}

/// Width and height of the key block with a gap on every side.
pub fn block_size(cfg: &KeyboardConfig) -> (f32, f32) {
    let all = places(cfg);
    let boxes = all.iter().flatten();
    let right = boxes.clone().map(|p| p.x + p.w).fold(0.0, f32::max);
    let bottom = boxes.map(|p| p.y + p.h).fold(0.0, f32::max);
    (right + cfg.gap_px, bottom + cfg.gap_px)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kb() -> KeyboardConfig {
        KeyboardConfig {
            titles: Default::default(),
            key_px: 48.0,
            gap_px: 4.0,
            font_px: 16.0,
            guard_delay_ms: 0,
            guard_tries: 1,
            relabel_ms: 0,
            rows: vec![vec![0x29, 0x02], vec![0x0F, 0x10], vec![0x39, 0xE038]],
            widths: vec![(0x0F, 1.5), (0x39, 6.25)],
        }
    }

    #[test]
    fn places_follow_the_face_layout() {
        let p: Vec<(f32, f32, f32)> = places(&kb())
            .into_iter()
            .flatten()
            .map(|p| (p.x, p.y, p.w))
            .collect();
        assert_eq!(
            p,
            vec![
                (4.0, 4.0, 48.0),
                (56.0, 4.0, 48.0),
                (4.0, 56.0, 74.0),
                (82.0, 56.0, 48.0),
                (4.0, 108.0, 321.0),
                (329.0, 108.0, 48.0)
            ]
        );
        assert_eq!(block_size(&kb()), (381.0, 160.0));
    }
}
