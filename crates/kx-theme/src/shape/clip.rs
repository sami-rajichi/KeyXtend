//! The clipboard panel's sizes beside the shared panel's (`PanelShape`): its filters' padding, tick box, rows and
//! buttons.

use serde::{Deserialize, Serialize};

/// The clipboard panel (mock-up `.clip`, `buildClip`); it grows with the keyboard.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClipShape {
    /// The header's kept line size (`.sm`).
    pub kept_px: f32,
    /// Padding around the filters: top, sides, bottom (`.filters`).
    pub filters_pad_px: [f32; 3],
    /// The tick box (`.ck`): side, radius, border, number size.
    pub tick_px: [f32; 4],
    /// The tick number's weight.
    pub tick_weight: u16,
    /// The picture's inset in its tile.
    pub thumb_inset_px: f32,
    /// The pin icon (`.pin .i`).
    pub pin_px: f32,
    /// The expires chip (`.exp`): text size, padding top and bottom, sides.
    pub expires_px: [f32; 3],
    /// Buttons (`.btn`): height, radius, side padding, text size, icon, icon gap.
    pub button_px: [f32; 6],
    /// The buttons' text weight.
    pub button_weight: u16,
    /// Gap between the footer's buttons (`.lfoot`).
    pub foot_gap_px: f32,
    /// The note band: padding, radius, text size.
    pub note_px: [f32; 3],
    /// The storage line: padding top and bottom, text size, icon.
    pub room_px: [f32; 3],
    /// The preview's text size: Latin, Arabic.
    pub preview_px: [f32; 2],
}
