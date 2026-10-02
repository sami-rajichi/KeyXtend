//! `shape.toml`: sizes the three themes share, from the mock-up's structure CSS, in pixels at size 1.

use serde::{Deserialize, Serialize};

use crate::extras::{
    CaptionShape, ListShape, MenuShape, NoticeShape, PanelShape, PillShape, RingShape, SnipShape,
    TrayShape,
};

mod check;

/// Top-bar buttons (`.tb`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BarShape {
    /// Button width and height.
    pub button_px: [f32; 2],
    /// Icon size.
    pub icon_px: f32,
    /// Text size of the S, M and L buttons.
    pub label_px: f32,
    /// Their weight while another size is shown.
    pub weight: u16,
    /// Their weight while their size is shown.
    pub lit_weight: u16,
    /// Corner radius as a share of the key radius.
    pub radius_share: f32,
    /// Gap between items.
    pub gap_px: f32,
    /// The lit size's mark under its letter: width, height and gap above the button's bottom.
    pub mark_px: [f32; 3],
}

/// Suggestion chips (`.chip`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChipShape {
    /// Height.
    pub height_px: f32,
    /// Side padding.
    pub pad_px: f32,
    /// Text size, Latin and Arabic.
    pub font_px: [f32; 2],
    /// Text weight.
    pub weight: u16,
    /// Icon size.
    pub icon_px: f32,
    /// Gap between icon and text.
    pub gap_px: f32,
}

/// Legends, icons and texts on keys (`.lg`, `.lg2`, `.lg3`, `.ico`, `.sp`, `.sub`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LegendShape {
    /// Second and third legend sizes as shares of the main one.
    pub corner_scale: [f32; 2],
    /// Corner legends' distance from the top or bottom, and from the side.
    pub corner_px: [f32; 2],
    /// Weight of the corner legends.
    pub corner_weight: u16,
    /// Icon size on main and side keys.
    pub icon_px: [f32; 2],
    /// Gap between an icon and a text beside it.
    pub gap_px: f32,
    /// Space bar text size, Latin and Arabic.
    pub space_px: [f32; 2],
    /// Space bar side padding.
    pub space_pad_px: f32,
    /// Side padding a named key's label keeps on its cap.
    pub label_pad_px: f32,
    /// The smallest size a label shrinks to, not scaled with the keyboard.
    pub label_min_px: f32,
}

/// The LED dot (`.led`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LedShape {
    /// Diameter.
    pub size_px: f32,
    /// Distance from the cap's top and right edges.
    pub inset_px: f32,
}

/// The language key's carousel (`.car`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LangShape {
    /// Gap between names.
    pub gap_px: f32,
    /// Size and opacity of the side names.
    pub side: [f32; 2],
    /// Weight of the side names.
    pub side_weight: u16,
    /// Current name size, Latin and Arabic.
    pub font_px: [f32; 2],
    /// Current name weight.
    pub weight: u16,
    /// A click on this share of the key, from the left, goes back a layout.
    pub back_share: f32,
    /// Gap between the Cycle and List designs' icon, name and dots (`.lk`).
    pub icon_gap_px: f32,
    /// The Cycle design.
    pub cycle: CycleShape,
    /// The List design.
    pub list: ListKeyShape,
}

/// The language key's Cycle design (`.cyc`, `.dots`): the globe, the name, and one dot per layout.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CycleShape {
    /// Globe icon size.
    pub icon_px: f32,
    /// Globe icon opacity.
    pub icon_share: f32,
    /// Dot diameter.
    pub dot_px: f32,
    /// The lit dot's diameter, larger so it stands out from the unlit ones.
    pub dot_on_px: f32,
    /// Gap between dots.
    pub dot_gap_px: f32,
    /// Opacity of a dot whose layout is not shown.
    pub dot_off: f32,
}

/// The language key's List design (`.lst`): the name and a chevron that opens the list.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ListKeyShape {
    /// Chevron size.
    pub icon_px: f32,
    /// Chevron opacity.
    pub icon_share: f32,
}

/// The D-pad (`.dpad`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DpadShape {
    /// Gap between the four arrows.
    pub gap_px: f32,
    /// Arrow icon size.
    pub icon_px: f32,
    /// Arrow icon padding, as a share of the pad.
    pub icon_pad: f32,
    /// Hub inset, as a share of the pad.
    pub hub: f32,
    /// Stop button inset, as a share of the pad.
    pub stop: f32,
    /// Stop icon size.
    pub stop_icon_px: f32,
    /// Turn of the arrow grid, in degrees.
    pub turn_deg: f32,
}

/// Pointer feedback (`.down`) and the move ring.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PressShape {
    /// A pressed cap moves down this far.
    pub sink_px: f32,
    /// And shrinks to this share.
    pub scale: f32,
    /// The ring around a keyboard being moved or resized.
    pub ring_px: f32,
}

/// Tooltips with a button's name.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TipShape {
    /// Text size.
    pub font_px: f32,
    /// Text weight.
    pub weight: u16,
    /// Padding at the top and bottom, and at the sides.
    pub pad_px: [f32; 2],
    /// Corner radius.
    pub radius_px: f32,
    /// Gap between the tooltip and its button.
    pub gap_px: f32,
}

/// The resize corner and test strip (not in the mock-up), and the minimise bubble.
#[allow(
    clippy::struct_field_names,
    reason = "every shape size names its unit, pixels here"
)]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraShape {
    /// Resize corner diameter and icon size.
    pub corner_px: [f32; 2],
    /// Bubble diameter, icon size, and inset from the screen corner (mock-up `.kb-bubble`).
    pub bubble_px: [f32; 3],
    /// Test strip height, gap above it, padding and text size.
    pub strip_px: [f32; 4],
}

/// Every shared size.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    /// Top-bar buttons.
    pub bar: BarShape,
    /// Suggestion chips.
    pub chip: ChipShape,
    /// Key legends and texts.
    pub legend: LegendShape,
    /// The LED.
    pub led: LedShape,
    /// The language key.
    pub lang: LangShape,
    /// The D-pad.
    pub dpad: DpadShape,
    /// Pointer feedback.
    pub press: PressShape,
    /// Tooltips.
    pub tip: TipShape,
    /// The resize corner, the bubble and the test strip.
    pub extra: ExtraShape,
    /// Hairline borders.
    pub line: LineShape,
    /// The Copy pill.
    pub pill: PillShape,
    /// The caption bar.
    pub caption: CaptionShape,
    /// The snip overlay.
    pub snip: SnipShape,
    /// The Arabic test panel.
    pub panel: PanelShape,
    /// The hold ring.
    pub ring: RingShape,
    /// Keys that come in a later version.
    pub dim: DimShape,
    /// The hover outline in high contrast.
    pub hc_outline: OutlineShape,
    /// The plate over a system backdrop.
    pub backdrop: BackdropShape,
    /// The notice banner.
    pub notice: NoticeShape,
    /// The tray menu.
    pub menu: MenuShape,
    /// The language list panel.
    pub list: ListShape,
    /// The tray icon's drawing.
    pub tray: TrayShape,
    /// What the drawn check measures caps by.
    pub drawn: DrawnShape,
}

/// What the drawn check measures caps by.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrawnShape {
    /// The keyboard's row height at size 1; a cap is this less the theme's gap.
    pub row_px: f32,
}

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

/// The checked sizes in `text`.
pub(crate) fn parse(text: &str) -> Result<Shape, String> {
    let s: Shape = toml::from_str(text).map_err(|e| e.to_string())?;
    s.check()?;
    Ok(s)
}

#[cfg(test)]
mod tests;
