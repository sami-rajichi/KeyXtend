//! Where each key sits, in logical pixels: one calculation shared by every face and the harness.

use serde::{Deserialize, Serialize};

use crate::config::{KeyboardConfig, SpikeConfig, ToolButton};
use crate::layout::width;

/// One key's box, from the top-left corner of the window's client area.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Place {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub w: f32,
    /// Height.
    pub h: f32,
}

impl Place {
    /// Left, top, right and bottom in physical pixels at scale `dpr`, rounded outwards so the box is covered.
    pub fn outward(&self, dpr: f32) -> [i32; 4] {
        [
            (self.x * dpr).floor() as i32,
            (self.y * dpr).floor() as i32,
            ((self.x + self.w) * dpr).ceil() as i32,
            ((self.y + self.h) * dpr).ceil() as i32,
        ]
    }
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

/// `n` tool buttons sharing one key-high row under the block, with a gap around each.
pub fn tools(cfg: &KeyboardConfig, n: usize) -> Vec<Place> {
    let ((block_w, block_h), gap) = (block_size(cfg), cfg.gap_px);
    let w = (block_w - gap * (n as f32 + 1.0)) / n.max(1) as f32;
    (0..n)
        .map(|i| Place {
            x: gap + i as f32 * (w + gap),
            y: block_h,
            w,
            h: cfg.key_px,
        })
        .collect()
}

/// Each tool button's label and box, in row order.
pub fn tool_row(cfg: &SpikeConfig) -> Vec<(String, Place)> {
    let boxes = tools(&cfg.keyboard, ToolButton::ALL.len());
    ToolButton::ALL
        .iter()
        .zip(boxes)
        .map(|(&b, p)| (cfg.tools.labels.of(b).to_string(), p))
        .collect()
}

/// Width and height of the keys plus the tools row, without the status line.
pub fn face_size(cfg: &KeyboardConfig) -> (f32, f32) {
    let (w, h) = block_size(cfg);
    (w, h + cfg.key_px + cfg.gap_px)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outward_covers_the_box_in_whole_physical_pixels() {
        let p = Place {
            x: 0.5,
            y: 1.0,
            w: 10.0,
            h: 2.0,
        };
        assert_eq!(p.outward(1.25), [0, 1, 14, 4]);
        assert_eq!(p.outward(1.0), [0, 1, 11, 3]);
    }

    #[test]
    fn places_arrive_from_qml_as_json() {
        let all: Vec<Place> = serde_json::from_str(r#"[{"x":1,"y":2,"w":3,"h":4}]"#).expect("json");
        assert_eq!(all[0].outward(2.0), [2, 4, 8, 12]);
    }

    fn kb() -> KeyboardConfig {
        KeyboardConfig {
            title: String::new(),
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

    #[test]
    fn tools_share_one_row_under_the_block() {
        let t = tools(&kb(), 3);
        let close = |a: f32, b: f32| (a - b).abs() < 0.01;
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
