//! Sizes that apply across the look: hairlines, dimmed keys, the high-contrast outline and the plate over a backdrop.

use serde::{Deserialize, Serialize};

/// Hairlines in `pop_line`: panel and tooltip edges, the corner's rim and dividers.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineShape {
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

/// The hover outline in high contrast, drawn in Highlight inside the key.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutlineShape {
    /// Width, not scaled with the keyboard.
    pub px: f32,
}

/// The plate over a system backdrop (`[ui] backdrop`), so the material shows through.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackdropShape {
    /// The share of its colour the plate keeps.
    pub plate: f32,
}
