//! Sizes of the small windows beside the keyboard: the Copy pill, the caption bar, the snip overlay, the Arabic panel
//! and the hold ring.

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
    /// The Copy icon's size.
    pub icon_px: f32,
    /// Gap between the icon and the text.
    pub gap_px: f32,
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
    /// The status and words icons' size.
    pub icon_px: f32,
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

/// The Arabic test panel (mock-up `.pop`, `.pop-h`, `.lrow`, `.pg`, `.lfoot`); it grows with the keyboard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PanelShape {
    /// Width.
    pub width_px: f32,
    /// Corner radius as a share of the window radius.
    pub radius_share: f32,
    /// Header padding: top, end, bottom, start.
    pub head_pad_px: [f32; 4],
    /// Gap between the header's icon, title and close button.
    pub head_gap_px: f32,
    /// Title size.
    pub title_px: f32,
    /// Title weight.
    pub title_weight: u16,
    /// Icon buttons (`.ib`): side, corner radius, icon size, which the header's icon shares.
    pub button_px: [f32; 3],
    /// Padding around the list: top, sides, bottom.
    pub body_pad_px: [f32; 3],
    /// Gap between the list and the pager.
    pub body_gap_px: f32,
    /// Rows (`.lrow`): height, gap, side padding, corner radius.
    pub row_px: [f32; 4],
    /// Row text size: Latin, Arabic.
    pub row_text_px: [f32; 2],
    /// Page buttons (`.pg`): width, height, corner radius, icon size.
    pub page_px: [f32; 4],
    /// Gap between the page buttons and the page number.
    pub pager_gap_px: f32,
    /// Opacity of a page button that cannot be used (`.pg[disabled]`).
    pub off_share: f32,
    /// Page number size (`.pgn`).
    pub count_px: f32,
    /// Page number weight.
    pub count_weight: u16,
    /// Footer padding (`.lfoot`): top and bottom, sides.
    pub foot_pad_px: [f32; 2],
    /// Text field: height, corner radius, side padding, text size.
    pub field_px: [f32; 4],
    /// Gap to the keyboard, and the least gap to the screen edge (mock-up `placePop`).
    pub gap_px: [f32; 2],
}

/// The hold ring around the pointer and its burst (mock-up `.ring`, `.burst`); it does not grow with the keyboard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RingShape {
    /// Side of the ring and of its window.
    pub box_px: f32,
    /// Border of the first and the second wave.
    pub wave_px: [f32; 2],
    /// The core's side as a share of the ring's.
    pub core_share: f32,
    /// The burst ring's side and border.
    pub burst_px: [f32; 2],
    /// A burst dot's side.
    pub dot_px: f32,
    /// How many dots fly out, evenly round.
    pub dots: u16,
    /// How far from the centre the dots start and end.
    pub fly_px: [f32; 2],
}
