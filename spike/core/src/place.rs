//! The stage-1 key block and tools row, in logical pixels, for the gates (G2 to G4) and the tools row until they move.
//! Boxes are kx-mod-keyboard's `Area`, which replaced this file's `Place` in P3 (ADR-0014).

use kx_mod_keyboard::Area;

use crate::config::{KeyboardConfig, SpikeConfig, ToolButton};

/// Width of a key that `widths` in spike.toml does not list, in key units.
pub const DEFAULT_WIDTH: f32 = 1.0;

/// Width of `code` in key units, from `widths` in spike.toml.
pub fn width(cfg: &KeyboardConfig, code: u32) -> f32 {
    cfg.widths
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(DEFAULT_WIDTH, |(_, w)| *w)
}

/// Every key's box, row by row: the block starts at (gap, gap); a key `u` units wide takes `u` pitches.
pub fn places(cfg: &KeyboardConfig) -> Vec<Vec<Area>> {
    let (key, gap) = (f64::from(cfg.key_px), f64::from(cfg.gap_px));
    let pitch = key + gap;
    cfg.rows
        .iter()
        .enumerate()
        .map(|(r, row)| {
            let mut x = gap;
            row.iter()
                .map(|&code| {
                    let u = f64::from(width(cfg, code));
                    let place = Area {
                        x,
                        y: gap + r as f64 * pitch,
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
pub fn block_size(cfg: &KeyboardConfig) -> (f64, f64) {
    let all = places(cfg);
    let boxes = all.iter().flatten();
    let right = boxes.clone().map(|p| p.x + p.w).fold(0.0, f64::max);
    let bottom = boxes.map(|p| p.y + p.h).fold(0.0, f64::max);
    let gap = f64::from(cfg.gap_px);
    (right + gap, bottom + gap)
}

/// `n` tool buttons sharing one key-high row under the block, with a gap around each.
pub fn tools(cfg: &KeyboardConfig, n: usize) -> Vec<Area> {
    let ((block_w, block_h), gap) = (block_size(cfg), f64::from(cfg.gap_px));
    let w = (block_w - gap * (n as f64 + 1.0)) / n.max(1) as f64;
    (0..n)
        .map(|i| Area {
            x: gap + i as f64 * (w + gap),
            y: block_h,
            w,
            h: f64::from(cfg.key_px),
        })
        .collect()
}

/// Each tool button's label and box, in row order.
pub fn tool_row(cfg: &SpikeConfig) -> Vec<(String, Area)> {
    let boxes = tools(&cfg.keyboard, ToolButton::ALL.len());
    ToolButton::ALL
        .iter()
        .zip(boxes)
        .map(|(&b, p)| (cfg.tools.labels.of(b).to_string(), p))
        .collect()
}

/// Width and height of the keys plus the tools row, without the status line.
pub fn face_size(cfg: &KeyboardConfig) -> (f64, f64) {
    let (w, h) = block_size(cfg);
    (w, h + f64::from(cfg.key_px) + f64::from(cfg.gap_px))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kb() -> KeyboardConfig {
        KeyboardConfig {
            title: String::new(),
            key_px: 48.0,
            gap_px: 4.0,
            font_px: 16.0,
            rows: vec![vec![0x29, 0x02], vec![0x0F, 0x10], vec![0x39, 0xE038]],
            widths: vec![(0x0F, 1.5), (0x39, 6.25)],
        }
    }

    #[test]
    fn places_follow_the_face_layout() {
        let p: Vec<(f64, f64, f64)> = places(&kb())
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

    #[test]
    fn width_uses_the_table_else_the_default() {
        assert_eq!(width(&kb(), 0x0F), 1.5);
        assert_eq!(width(&kb(), 0x10), DEFAULT_WIDTH);
    }

    #[test]
    fn tools_share_one_row_under_the_block() {
        let t = tools(&kb(), 3);
        let close = |a: f64, b: f64| (a - b).abs() < 0.01;
        assert!(
            t.iter()
                .all(|p| p.y == 160.0 && p.h == 48.0 && close(p.w, t[0].w))
        );
        assert_eq!(t[0].x, 4.0);
        assert!(close(t[1].x, t[0].x + t[0].w + 4.0));
        let last = t[2];
        assert!(
            close(last.x + last.w + 4.0, 381.0),
            "the row fills the block"
        );
        assert_eq!(face_size(&kb()), (381.0, 212.0));
        assert!(tools(&kb(), 0).is_empty());
    }

    #[test]
    fn the_tool_row_pairs_each_label_with_its_box() {
        let cfg = crate::config::load().expect("spike.toml loads");
        let row = tool_row(&cfg);
        let boxes = tools(&cfg.keyboard, ToolButton::ALL.len());
        for (i, (label, at)) in row.iter().enumerate() {
            let b = ToolButton::at(i).expect("a button");
            assert_eq!((label.as_str(), *at), (cfg.tools.labels.of(b), boxes[i]));
        }
        assert_eq!(row.len(), ToolButton::ALL.len());
    }
}
