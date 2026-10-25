//! The board's rules, run whenever a board is read.

use super::{BoardError, BoardSpec, Role, RowItem};
use std::collections::HashSet;

impl BoardSpec {
    /// Refuses sizes that are not numbers above 0, rows that do not span the main block, a D-pad off the side grid,
    /// side keys off the grid, on the D-pad or sharing a cell, an id used twice, a role that does not fit its key,
    /// and a shortcuts layer that does not fit the side grid.
    ///
    /// # Errors
    /// The first broken rule.
    pub fn check(&self) -> Result<(), BoardError> {
        self.check_sizes()?;
        self.check_rows()?;
        self.check_dpad()?;
        self.check_side()?;
        self.check_ids()?;
        self.check_roles()?;
        self.check_layer()
    }

    /// Refuses a length that is not a number above 0 (a gap or pad may be 0), and empty grid counts.
    fn check_sizes(&self) -> Result<(), BoardError> {
        let above = [
            self.unit_px,
            self.row_px,
            self.main_units,
            self.side_unit,
            self.dpad.units,
        ];
        let gaps = [self.body_gap_px, self.top_bar_px, self.top_gap_px];
        let pads = self.plate_pad_px.iter().chain(&gaps);
        let ok = above.iter().all(|v| v.is_finite() && *v > 0.0)
            && pads.into_iter().all(|v| v.is_finite() && *v >= 0.0)
            && self.columns_per_unit > 0
            && self.side_columns > 0
            && self.columns(self.main_units).is_some();
        if ok { Ok(()) } else { Err(BoardError::Size) }
    }

    /// Refuses a named key that is not whole columns, and a row that does not span the main block.
    fn check_rows(&self) -> Result<(), BoardError> {
        let want = self.columns(self.main_units).map_or(0, u32::from);
        for (r, row) in self.rows.iter().enumerate() {
            let mut spans = 0u32;
            for item in row {
                spans = spans.saturating_add(self.item_columns(item)?);
            }
            if spans != want {
                let row = r + 1;
                return Err(BoardError::Row { row, spans, want });
            }
        }
        Ok(())
    }

    /// Grid columns `item` spans.
    fn item_columns(&self, item: &RowItem) -> Result<u32, BoardError> {
        match item {
            RowItem::Key(k) => self
                .columns(k.w)
                .map(u32::from)
                .ok_or_else(|| BoardError::Width(k.id.clone())),
            RowItem::Chars(c) => {
                let keys = u32::try_from(c.chars.len()).unwrap_or(u32::MAX);
                Ok(keys.saturating_mul(u32::from(self.columns_per_unit)))
            }
        }
    }

    /// True when side cell (`col`, `row`) is on the side grid.
    pub(super) fn on_grid(&self, col: u16, row: u16) -> bool {
        let rows = self.rows.len();
        (1..=self.side_columns).contains(&col) && row >= 1 && usize::from(row) <= rows
    }

    /// True when side cell (`col`, `row`) is under the D-pad.
    fn on_dpad(&self, col: u16, row: u16) -> bool {
        let [c0, r0, c1, r1] = self.dpad.cell;
        (c0..=c1).contains(&col) && (r0..=r1).contains(&row)
    }

    /// Refuses D-pad cells off the side grid or given last before first.
    fn check_dpad(&self) -> Result<(), BoardError> {
        let [c0, r0, c1, r1] = self.dpad.cell;
        let ok = c0 <= c1 && r0 <= r1 && self.on_grid(c0, r0) && self.on_grid(c1, r1);
        if ok { Ok(()) } else { Err(BoardError::Dpad) }
    }

    /// Refuses side keys off the grid, on the D-pad or in a cell already taken.
    fn check_side(&self) -> Result<(), BoardError> {
        let mut taken = HashSet::new();
        for s in &self.side {
            let free = taken.insert((s.col, s.row));
            if !free || !self.on_grid(s.col, s.row) || self.on_dpad(s.col, s.row) {
                return Err(BoardError::Side(s.id.clone()));
            }
        }
        Ok(())
    }

    /// Refuses an id that two keys share, character keys included.
    fn check_ids(&self) -> Result<(), BoardError> {
        let mut seen = HashSet::new();
        for (id, ..) in self.roles() {
            if seen.contains(&id) {
                return Err(BoardError::Twice(id));
            }
            seen.insert(id);
        }
        Ok(())
    }

    /// Refuses a named or side key whose role is `char` or one only the layer generates, a main-row key that shows the
    /// layer or asks for the power dialog, a key that sends a scan code when its role sends none, or the reverse, and a
    /// key that repeats or tells but is not `send`.
    fn check_roles(&self) -> Result<(), BoardError> {
        let named = self.rows.iter().flatten().filter_map(|i| match i {
            RowItem::Key(k) => {
                let send_only = k.repeat || k.tell.is_some();
                Some((&k.id, k.role, k.sc.is_some(), send_only, false))
            }
            RowItem::Chars(_) => None,
        });
        let side = self
            .side
            .iter()
            .map(|s| (&s.id, s.role, false, false, true));
        for (id, role, has_code, send_only, on_side) in named.chain(side) {
            let stray = send_only && role != Role::Send;
            let made = matches!(role, Role::Char | Role::Shortcut | Role::Next);
            let side_only = !on_side && matches!(role, Role::Layer | Role::Power);
            if made || side_only || role.sends() != has_code || stray {
                return Err(BoardError::Role(id.clone()));
            }
        }
        Ok(())
    }
}
