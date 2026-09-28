//! Sizes of the small windows beside the keyboard: the Copy pill, the caption bar and the snip overlay.

use serde::{Deserialize, Serialize};

/// The Copy pill (mock-up `.selpill`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PillShape {
    /// Room between the pill's edge and its button.
    pub pad_px: f32,
    /// Button padding at the sides; the height is `tools.pill_px` in spike.toml.
    pub button_side_px: f32,
    /// Text size.
    pub font_px: f32,
    /// Text weight.
    pub weight: u16,
}

/// The caption bar (mock-up `.capbar`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaptionShape {
    /// Padding around a status: top and bottom, left, right.
    pub pad_px: [f32; 3],
    /// Padding around the words heard (mock-up `.capbar.done`).
    pub words_pad_px: f32,
    /// Gap between the dot and the status text.
    pub gap_px: f32,
    /// Status text size, such as "Listening…".
    pub status_px: f32,
    /// Status text weight.
    pub status_weight: u16,
    /// Size of the words heard: Latin, Arabic.
    pub words_px: [f32; 2],
    /// The recording dot's diameter.
    pub dot_px: f32,
    /// Height of the bar while it shows a status.
    pub bar_px: f32,
    /// Corner radius around the words heard (mock-up `.capbar.done`).
    pub words_radius_px: f32,
    /// Shimmer bar height and corner radius (mock-up `.shimmer`).
    pub shimmer_px: [f32; 2],
    /// The shimmer's light band repeats every this many bar widths.
    pub shimmer_span: f32,
}

/// The snip overlay (mock-up `.snip-rect .dims` and `.snip-bar`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SnipShape {
    /// Size tag text size.
    pub tag_px: f32,
    /// Size tag text weight.
    pub tag_weight: u16,
    /// Size tag padding: top and bottom, sides.
    pub tag_pad_px: [f32; 2],
    /// Size tag corner radius.
    pub tag_radius_px: f32,
    /// How far the size tag sits above the region.
    pub tag_gap_px: f32,
    /// Gap between the hint bar and the top of the screen.
    pub bar_top_px: f32,
    /// Hint bar padding: top and bottom, sides.
    pub bar_pad_px: [f32; 2],
    /// Hint bar corner radius.
    pub bar_radius_px: f32,
    /// Hint text size.
    pub hint_px: f32,
}
