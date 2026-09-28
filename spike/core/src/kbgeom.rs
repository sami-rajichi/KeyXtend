//! The stage-2 keyboard's geometry from `spike.toml [layout]`: every key, the D-pad and the plate, at any size.

mod spec;

use crate::place::Place;
pub use spec::{CharSpec, KeyKind, KeySpec, LayoutConfig, RowItem, SideSpec};

/// One key: its id, kind, scan code, grid cell and the cap drawn inside it.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyBox {
    /// Id; character keys are `c-<row>-<index>`.
    pub id: String,
    /// Kind.
    pub kind: KeyKind,
    /// Scan code, if it sends one.
    pub sc: Option<u32>,
    /// The grid cell.
    pub cell: Place,
    /// The cap: the cell less half a gap on each side.
    pub cap: Place,
}

/// The whole keyboard at one size, from the plate's top-left corner.
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    /// The plate, which is the window.
    pub plate: Place,
    /// Main keys row by row, then side keys.
    pub keys: Vec<KeyBox>,
    /// The D-pad's round face.
    pub dpad: Place,
}

/// Where the blocks start, at size 1.
struct Frame {
    top: f32,
    main_x: f32,
    col_w: f32,
    side_x: f32,
    side_w: f32,
}

impl Frame {
    fn of(l: &LayoutConfig) -> Frame {
        let [pad_top, _, _, pad_left] = l.plate_pad_px;
        let side_x = pad_left + l.main_units * l.unit_px + l.body_gap_px;
        Frame {
            top: pad_top + l.top_bar_px + l.top_gap_px,
            main_x: pad_left,
            col_w: l.unit_px / l.columns_per_unit as f32,
            side_x,
            side_w: l.side_unit * l.unit_px,
        }
    }
}

/// `p` grown by `by` on every side; negative shrinks.
fn grow(p: Place, by: f32) -> Place {
    Place {
        x: p.x - by,
        y: p.y - by,
        w: p.w + 2.0 * by,
        h: p.h + 2.0 * by,
    }
}

/// `p` times `s`.
fn scale(p: Place, s: f32) -> Place {
    Place {
        x: p.x * s,
        y: p.y * s,
        w: p.w * s,
        h: p.h * s,
    }
}

/// A key in `cell`, with its cap half a gap inside.
fn key(id: String, kind: KeyKind, sc: Option<u32>, cell: Place, gap: f32) -> KeyBox {
    let cap = grow(cell, -gap / 2.0);
    KeyBox {
        id,
        kind,
        sc,
        cell,
        cap,
    }
}

/// The main rows' keys at size 1.
fn main_keys(l: &LayoutConfig, f: &Frame, gap: f32) -> Vec<KeyBox> {
    let mut keys = Vec::new();
    for (r, row) in l.rows.iter().enumerate() {
        let (mut col, y) = (0u32, f.top + r as f32 * l.row_px);
        let mut cell = |span: u32| {
            let p = Place {
                x: f.main_x + col as f32 * f.col_w,
                y,
                w: span as f32 * f.col_w,
                h: l.row_px,
            };
            col += span;
            p
        };
        for item in row {
            match item {
                RowItem::Key(k) => {
                    let span = l.columns(k.w).unwrap_or(l.columns_per_unit);
                    keys.push(key(k.id.clone(), k.kind, k.sc, cell(span), gap));
                }
                RowItem::Chars(c) => {
                    for (i, &sc) in c.chars.iter().enumerate() {
                        let id = format!("c-{r}-{i}");
                        keys.push(key(
                            id,
                            KeyKind::Char,
                            Some(sc),
                            cell(l.columns_per_unit),
                            gap,
                        ));
                    }
                }
            }
        }
    }
    keys
}

/// The side block's keys at size 1.
fn side_keys(l: &LayoutConfig, f: &Frame, gap: f32) -> Vec<KeyBox> {
    let cell = |col: u32, row: u32| Place {
        x: f.side_x + (col - 1) as f32 * f.side_w,
        y: f.top + (row - 1) as f32 * l.row_px,
        w: f.side_w,
        h: l.row_px,
    };
    let at = |s: &SideSpec| key(s.id.clone(), KeyKind::Act, None, cell(s.col, s.row), gap);
    l.side.iter().map(at).collect()
}

/// The D-pad face at size 1: centred in its cells, the smaller of its width and height less one gap.
fn dpad(l: &LayoutConfig, f: &Frame, gap: f32) -> Place {
    let [c0, r0, c1, r1] = l.dpad_cell;
    let rows = (r1 + 1 - r0) as f32;
    let wrap = Place {
        x: f.side_x + (c0 - 1) as f32 * f.side_w,
        y: f.top + (r0 - 1) as f32 * l.row_px,
        w: (c1 + 1 - c0) as f32 * f.side_w,
        h: rows * l.row_px,
    };
    let d = (l.dpad_units * l.unit_px).min(rows * l.row_px) - gap;
    Place {
        x: wrap.x + (wrap.w - d) / 2.0,
        y: wrap.y + (wrap.h - d) / 2.0,
        w: d,
        h: d,
    }
}

/// The keyboard with key gap `gap_px` at size `size`.
pub fn board(l: &LayoutConfig, gap_px: f32, size: f32) -> Board {
    let f = Frame::of(l);
    let [_, pad_right, pad_bottom, _] = l.plate_pad_px;
    let plate = Place {
        x: 0.0,
        y: 0.0,
        w: f.side_x + l.side_columns as f32 * f.side_w + pad_right,
        h: f.top + l.rows.len() as f32 * l.row_px + pad_bottom,
    };
    let mut keys = main_keys(l, &f, gap_px);
    keys.extend(side_keys(l, &f, gap_px));
    for k in &mut keys {
        k.cell = scale(k.cell, size);
        k.cap = scale(k.cap, size);
    }
    Board {
        plate: scale(plate, size),
        keys,
        dpad: scale(dpad(l, &f, gap_px), size),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Allowed rounding error in logical pixels.
    const EPS: f32 = 0.01;

    fn cfg() -> crate::config::SpikeConfig {
        crate::config::load().expect("spike.toml loads")
    }

    fn inside(outer: &Place, inner: &Place) -> bool {
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.x + inner.w <= outer.x + outer.w + EPS
            && inner.y + inner.h <= outer.y + outer.h + EPS
    }

    #[test]
    fn the_mock_up_board_is_about_671_by_196_at_size_1() {
        let b = board(&cfg().layout, 4.0, 1.0);
        assert!((b.plate.w - 670.78).abs() < EPS, "{}", b.plate.w);
        assert!((b.plate.h - 196.0).abs() < EPS, "{}", b.plate.h);
        assert_eq!(
            b.keys.iter().filter(|k| k.kind == KeyKind::Char).count(),
            47
        );
        assert_eq!(b.keys.iter().filter(|k| k.kind == KeyKind::Act).count(), 9);
    }

    #[test]
    fn every_row_spans_the_main_block_and_a_bad_row_is_refused() {
        let mut l = cfg().layout;
        assert_eq!(l.check(), Ok(()));
        l.rows[2].push(RowItem::Chars(CharSpec { chars: vec![0x56] }));
        let e = l.check().expect_err("row 3 is too long");
        assert!(e.contains("row 3"), "{e}");
    }

    #[test]
    fn a_side_key_on_the_dpad_is_refused() {
        let mut l = cfg().layout;
        l.side[0].row = 2;
        let e = l.check().expect_err("rclick sits on the D-pad");
        assert!(e.contains("rclick"), "{e}");
    }

    #[test]
    fn a_bad_dpad_cell_or_empty_grid_is_refused() {
        for cell in [[0, 2, 2, 4], [2, 2, 1, 4], [1, 4, 2, 2], [1, 2, 9, 4]] {
            let mut l = cfg().layout;
            l.dpad_cell = cell;
            assert!(l.check().is_err(), "{cell:?}");
        }
        let mut l = cfg().layout;
        l.columns_per_unit = 0;
        assert!(l.check().is_err(), "no columns");
    }

    #[test]
    fn two_side_keys_in_one_cell_are_refused() {
        let mut l = cfg().layout;
        l.side[1].col = l.side[0].col;
        l.side[1].row = l.side[0].row;
        assert!(l.check().is_err());
    }

    #[test]
    fn keys_and_dpad_stay_inside_the_plate_at_the_size_limits() {
        let c = cfg();
        for gap in [3.0, 4.0, 5.0] {
            for size in [c.size.min, 1.0, c.size.max] {
                let b = board(&c.layout, gap, size);
                assert!(
                    b.keys.iter().all(|k| inside(&b.plate, &k.cap)),
                    "{gap} {size}"
                );
                assert!(inside(&b.plate, &b.dpad), "{gap} {size}");
                assert!(b.keys.iter().all(|k| inside(&k.cell, &k.cap)));
            }
        }
    }

    #[test]
    fn the_dpad_is_the_smaller_of_its_width_and_height_less_a_gap() {
        let l = cfg().layout;
        let b = board(&l, 4.0, 1.0);
        let [_, r0, _, r1] = l.dpad_cell;
        let rows = (r1 + 1 - r0) as f32;
        let d = (l.dpad_units * l.unit_px).min(rows * l.row_px) - 4.0;
        assert!(
            (b.dpad.w - d).abs() < EPS && (b.dpad.h - d).abs() < EPS,
            "{:?}",
            b.dpad
        );
    }

    #[test]
    fn a_larger_size_scales_every_box() {
        let l = cfg().layout;
        let (one, big) = (board(&l, 4.0, 1.0), board(&l, 4.0, 1.5));
        assert!((big.plate.w - one.plate.w * 1.5).abs() < EPS);
        let (a, b) = (&one.keys[20].cap, &big.keys[20].cap);
        assert!((b.x - a.x * 1.5).abs() < EPS && (b.w - a.w * 1.5).abs() < EPS);
    }
}
