//! Quick-fill's sizes beside the shared panel's (`PanelShape`) and the clipboard's buttons: its width, the page's lines
//! over its rows, an own label's line, the mask and New's kind chips.

use serde::{Deserialize, Serialize};

/// Quick-fill (design "The panel"; mock-up `.fill`, `buildFill`); it grows with the keyboard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FillShape {
    /// The panel's width.
    pub width_px: f32,
    /// Over the rows: the gap above the first line, between the lines, and below the second.
    pub top_px: [f32; 3],
    /// An own label's share of its field's line.
    pub own_share: f32,
    /// The mask's letter spacing, as a share of its size (`.masked`).
    pub mask_em: f32,
    /// New's kind chips: gap, radius, icon, gap to the label.
    pub kind_px: [f32; 4],
    /// How many rows New's kind chips take.
    pub kind_rows: u8,
}
