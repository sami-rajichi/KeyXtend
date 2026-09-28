//! `spike.toml [layout]` as written: main rows, side keys and the D-pad cells, with the check run at load.

use serde::Deserialize;

/// What a key is, which sets its colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyKind {
    /// A character key.
    Char,
    /// A modifier or navigation key.
    Mod,
    /// Esc or Backspace.
    Danger,
    /// Enter.
    Enter,
    /// The language key.
    Lang,
    /// The space bar.
    Space,
    /// A side-block action key.
    Act,
}

/// A named key in a main row.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeySpec {
    /// Short id, such as `esc`.
    pub id: String,
    /// Width in key units.
    pub w: f32,
    /// Kind.
    pub kind: KeyKind,
    /// Scan code it sends; `0xE0xx` is extended; none for the language key.
    #[serde(default)]
    pub sc: Option<u32>,
}

/// A run of 1-unit character keys, by scan code.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharSpec {
    /// Scan codes, left to right.
    pub chars: Vec<u32>,
}

/// One entry of a main row.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RowItem {
    /// A named key.
    Key(KeySpec),
    /// Character keys.
    Chars(CharSpec),
}

/// A side key and its cell, 1-based.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SideSpec {
    /// Short id, such as `mic`.
    pub id: String,
    /// Column, from 1.
    pub col: u32,
    /// Row, from 1.
    pub row: u32,
}

/// `spike.toml [layout]`, in logical pixels at size 1.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutConfig {
    /// One key unit wide.
    pub unit_px: f32,
    /// One row high.
    pub row_px: f32,
    /// Main block width in units.
    pub main_units: f32,
    /// Grid columns per unit.
    pub columns_per_unit: u32,
    /// Side block columns.
    pub side_columns: u32,
    /// Side column width in units.
    pub side_unit: f32,
    /// Gap between the blocks.
    pub body_gap_px: f32,
    /// Plate padding: top, right, bottom, left.
    pub plate_pad_px: [f32; 4],
    /// Top bar height.
    pub top_bar_px: f32,
    /// Gap under the top bar.
    pub top_gap_px: f32,
    /// D-pad cells: first column, first row, last column, last row.
    pub dpad_cell: [u32; 4],
    /// Largest D-pad width in units.
    pub dpad_units: f32,
    /// Main rows, top to bottom.
    pub rows: Vec<Vec<RowItem>>,
    /// Side keys.
    pub side: Vec<SideSpec>,
}

impl LayoutConfig {
    /// Grid columns a key `w` units wide spans, if it is a whole number.
    pub(super) fn columns(&self, w: f32) -> Option<u32> {
        let span = w * self.columns_per_unit as f32;
        ((span - span.round()).abs() < f32::EPSILON && span >= 1.0).then(|| span.round() as u32)
    }

    /// Grid columns row item `i` spans.
    fn item_columns(&self, i: &RowItem) -> Result<u32, String> {
        match i {
            RowItem::Key(k) => self
                .columns(k.w)
                .ok_or_else(|| format!("key {} is not a whole number of columns", k.id)),
            RowItem::Chars(c) => Ok(c.chars.len() as u32 * self.columns_per_unit),
        }
    }

    /// True when side cell (`col`, `row`) is under the D-pad.
    fn on_dpad(&self, col: u32, row: u32) -> bool {
        let [c0, r0, c1, r1] = self.dpad_cell;
        (c0..=c1).contains(&col) && (r0..=r1).contains(&row)
    }

    /// Refuses an empty grid, rows that do not fill the main block, a D-pad off the side grid,
    /// and side keys off the grid, on the D-pad or sharing a cell.
    pub fn check(&self) -> Result<(), String> {
        if self.columns_per_unit == 0 || self.side_columns == 0 {
            return Err("layout: columns_per_unit and side_columns must be above 0".into());
        }
        self.check_rows()?;
        self.check_dpad()?;
        self.check_side()
    }

    /// Refuses rows that do not span the main block.
    fn check_rows(&self) -> Result<(), String> {
        let want = (self.main_units * self.columns_per_unit as f32).round() as u32;
        for (r, row) in self.rows.iter().enumerate() {
            let cols = row
                .iter()
                .map(|i| self.item_columns(i))
                .sum::<Result<u32, _>>()?;
            if cols != want {
                return Err(format!(
                    "layout row {} spans {cols} of {want} columns",
                    r + 1
                ));
            }
        }
        Ok(())
    }

    /// True when side cell (`col`, `row`) is on the side grid.
    fn on_grid(&self, col: u32, row: u32) -> bool {
        let rows = self.rows.len() as u32;
        (1..=self.side_columns).contains(&col) && (1..=rows).contains(&row)
    }

    /// Refuses D-pad cells off the side grid or given last before first.
    fn check_dpad(&self) -> Result<(), String> {
        let [c0, r0, c1, r1] = self.dpad_cell;
        if c0 > c1 || r0 > r1 || !self.on_grid(c0, r0) || !self.on_grid(c1, r1) {
            return Err("layout: dpad_cell must be first col, first row, last col, last row on the side grid".into());
        }
        Ok(())
    }

    /// Refuses side keys off the grid, on the D-pad or in a cell already taken.
    fn check_side(&self) -> Result<(), String> {
        for (i, s) in self.side.iter().enumerate() {
            let twice = self.side[..i]
                .iter()
                .any(|t| (t.col, t.row) == (s.col, s.row));
            if twice || !self.on_grid(s.col, s.row) || self.on_dpad(s.col, s.row) {
                return Err(format!(
                    "side key {} is off the grid, on the D-pad or in a taken cell",
                    s.id
                ));
            }
        }
        Ok(())
    }
}
