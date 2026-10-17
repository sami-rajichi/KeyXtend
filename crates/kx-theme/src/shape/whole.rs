//! Sizes that apply across the look: line widths, dimmed keys and the plate over a backdrop.

use serde::{Deserialize, Serialize};

/// A line's width: hairlines in `pop_line`, the hover outline in high contrast and the scroll target's outline.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WidthShape {
    /// Width, not scaled with the keyboard.
    pub px: f32,
}

/// Keys that come in a later version (`.btn[disabled]`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DimShape {
    /// Opacity over the plate, from which each palette's `dim` and `dim_ink` come.
    pub share: f32,
}

/// The plate over a system backdrop (`[ui] backdrop`), so the material shows through.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackdropShape {
    /// The share of its colour the plate keeps.
    pub plate: f32,
}
