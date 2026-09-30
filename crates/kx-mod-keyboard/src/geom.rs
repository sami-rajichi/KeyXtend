//! The board at any size: every key's cell and cap, the D-pad and the top bar, in logical pixels.

use crate::spec::{BoardSpec, KeyKind, Role, RowKey, SideSpec};
use kx_module_api::ports::Scan;
use serde::{Deserialize, Serialize};

/// A box in logical pixels, from the plate's top-left corner.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Area {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl Area {
    /// This box grown by `by` on every side; a negative `by` shrinks it.
    #[must_use]
    pub fn grow(self, by: f64) -> Self {
        Self {
            x: self.x - by,
            y: self.y - by,
            w: self.w + 2.0 * by,
            h: self.h + 2.0 * by,
        }
    }

    /// This box times `k`.
    #[must_use]
    pub fn scale(self, k: f64) -> Self {
        Self {
            x: self.x * k,
            y: self.y * k,
            w: self.w * k,
            h: self.h * k,
        }
    }
}

/// One key: its id, kind, role, scan code and row, its grid cell and the cap drawn inside it.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyBox {
    /// Id; character keys are `c-<row>-<index>`.
    pub id: String,
    /// Kind, which sets its colours.
    pub kind: KeyKind,
    /// What a click does.
    pub role: Role,
    /// The scan code it sends, if any.
    pub sc: Option<Scan>,
    /// Its row, from 0 at the top.
    pub row: usize,
    /// The grid cell.
    pub cell: Area,
    /// The cap: the cell less half a gap on each side.
    pub cap: Area,
}

/// The whole keyboard at one size, from the plate's top-left corner.
#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    /// The plate, which is the window.
    pub plate: Area,
    /// Main keys row by row, then side keys.
    pub keys: Vec<KeyBox>,
    /// The D-pad's round face.
    pub dpad: Area,
    /// The top bar.
    pub bar: Area,
}

/// Where the blocks start, at size 1.
struct Frame {
    top: f64,
    main_x: f64,
    col_w: f64,
    side_x: f64,
    side_w: f64,
}

impl Frame {
    fn of(s: &BoardSpec) -> Self {
        let [pad_top, _, _, pad_left] = s.plate_pad_px;
        Self {
            top: pad_top + s.top_bar_px + s.top_gap_px,
            main_x: pad_left,
            col_w: s.unit_px / f64::from(s.columns_per_unit),
            side_x: pad_left + s.main_units * s.unit_px + s.body_gap_px,
            side_w: s.side_unit * s.unit_px,
        }
    }
}

/// What a key is before it has a place.
struct Named {
    id: String,
    kind: KeyKind,
    role: Role,
    sc: Option<Scan>,
    row: usize,
}

/// Key `n` in `cell`, with its cap half a gap inside.
fn key(n: Named, cell: Area, gap: f64) -> KeyBox {
    let Named {
        id,
        kind,
        role,
        sc,
        row,
    } = n;
    KeyBox {
        id,
        kind,
        role,
        sc,
        row,
        cell,
        cap: cell.grow(-gap / 2.0),
    }
}

/// `n` as a float; a count past `u32` stops there.
fn float(n: usize) -> f64 {
    u32::try_from(n).map_or(f64::from(u32::MAX), f64::from)
}

/// Main row `r`'s keys as named keys, each with its width in grid columns.
/// Only checked specs reach it, so every width is whole columns; one unit only stands in for a broken one.
fn row_named(s: &BoardSpec, r: usize) -> Vec<(Named, u16)> {
    let one = s.columns_per_unit;
    let named = |k: RowKey<'_>| match k {
        RowKey::Named(k) => {
            let n = Named {
                id: k.id.clone(),
                kind: k.kind,
                role: k.role,
                sc: k.sc,
                row: r,
            };
            (n, s.columns(k.w).unwrap_or(one))
        }
        RowKey::Char { id, sc } => {
            let n = Named {
                id,
                kind: KeyKind::Char,
                role: Role::Char,
                sc: Some(sc),
                row: r,
            };
            (n, one)
        }
    };
    s.row_keys(r).into_iter().map(named).collect()
}

/// The main rows' keys at size 1.
fn main_keys(s: &BoardSpec, f: &Frame, gap: f64) -> Vec<KeyBox> {
    let mut keys = Vec::new();
    for r in 0..s.rows.len() {
        let (mut col, y) = (0u32, f.top + float(r) * s.row_px);
        for (n, span) in row_named(s, r) {
            let cell = Area {
                x: f.main_x + f64::from(col) * f.col_w,
                y,
                w: f64::from(span) * f.col_w,
                h: s.row_px,
            };
            col += u32::from(span);
            keys.push(key(n, cell, gap));
        }
    }
    keys
}

/// Side cell (`col`, `row`), 1-based, at size 1.
fn side_cell(s: &BoardSpec, f: &Frame, col: u16, row: u16) -> Area {
    Area {
        x: f.side_x + f64::from(col.saturating_sub(1)) * f.side_w,
        y: f.top + f64::from(row.saturating_sub(1)) * s.row_px,
        w: f.side_w,
        h: s.row_px,
    }
}

/// The side block's keys at size 1.
fn side_keys(s: &BoardSpec, f: &Frame, gap: f64) -> Vec<KeyBox> {
    let at = |k: &SideSpec| {
        let n = Named {
            id: k.id.clone(),
            kind: KeyKind::Act,
            role: k.role,
            sc: None,
            row: usize::from(k.row.saturating_sub(1)),
        };
        key(n, side_cell(s, f, k.col, k.row), gap)
    };
    s.side.iter().map(at).collect()
}

/// The D-pad face at size 1: centred in its cells, the smaller of its width and height less one gap.
fn dpad(s: &BoardSpec, f: &Frame, gap: f64) -> Area {
    let [c0, r0, c1, r1] = s.dpad.cell;
    let wrap = side_cell(s, f, c0, r0);
    let cols = f64::from(c1) + 1.0 - f64::from(c0);
    let rows = f64::from(r1) + 1.0 - f64::from(r0);
    let (width, height) = (cols * f.side_w, rows * s.row_px);
    let side = (s.dpad.units * s.unit_px).min(height) - gap;
    Area {
        x: wrap.x + (width - side) / 2.0,
        y: wrap.y + (height - side) / 2.0,
        w: side,
        h: side,
    }
}

/// The keyboard with key gap `gap` at size `size`.
#[must_use]
pub fn board(spec: &BoardSpec, gap: f64, size: f64) -> Board {
    let f = Frame::of(spec);
    let [pad_top, pad_right, pad_bottom, pad_left] = spec.plate_pad_px;
    let plate = Area {
        x: 0.0,
        y: 0.0,
        w: f.side_x + f64::from(spec.side_columns) * f.side_w + pad_right,
        h: f.top + float(spec.rows.len()) * spec.row_px + pad_bottom,
    };
    let bar = Area {
        x: pad_left,
        y: pad_top,
        w: plate.w - pad_left - pad_right,
        h: spec.top_bar_px,
    };
    let mut keys = main_keys(spec, &f, gap);
    keys.extend(side_keys(spec, &f, gap));
    for k in &mut keys {
        k.cell = k.cell.scale(size);
        k.cap = k.cap.scale(size);
    }
    Board {
        plate: plate.scale(size),
        keys,
        dpad: dpad(spec, &f, gap).scale(size),
        bar: bar.scale(size),
    }
}

#[cfg(test)]
mod tests;
