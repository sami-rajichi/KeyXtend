//! `shape.toml`: sizes the three themes share, from the mock-up's structure CSS, in pixels at size 1.

use serde::{Deserialize, Serialize};

use super::FONT_WEIGHTS;
use super::extras::{CaptionShape, PillShape, SnipShape};
use crate::config::read_settings;

/// The shared sizes file, beside `spike.toml`.
const FILE: &str = "shape.toml";

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
    /// Their weight while their size is shown.
    pub lit_weight: u16,
    /// Corner radius as a share of the key radius.
    pub radius_share: f32,
    /// Gap between items.
    pub gap_px: f32,
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
    /// Current name size, Latin and Arabic.
    pub font_px: [f32; 2],
    /// Current name weight.
    pub weight: u16,
    /// A click on this share of the key, from the left, goes back a layout.
    pub back_share: f32,
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
}

/// Hairlines in `pop_line`: panel and tooltip edges, the corner's rim and dividers.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineShape {
    /// Width, not scaled with the keyboard.
    pub px: f32,
}

impl Shape {
    /// Refuses a weight outside 100 to 900, a share outside 0 to 1 and a shimmer span of 0 or less, naming the size.
    pub fn check(&self) -> Result<(), String> {
        let weights = [
            ("bar.lit_weight", self.bar.lit_weight),
            ("chip.weight", self.chip.weight),
            ("legend.corner_weight", self.legend.corner_weight),
            ("lang.weight", self.lang.weight),
            ("tip.weight", self.tip.weight),
            ("pill.weight", self.pill.weight),
            ("caption.status_weight", self.caption.status_weight),
            ("snip.tag_weight", self.snip.tag_weight),
        ];
        if let Some((n, w)) = weights.iter().find(|(_, w)| !FONT_WEIGHTS.contains(w)) {
            let (lo, hi) = (FONT_WEIGHTS.start(), FONT_WEIGHTS.end());
            return Err(format!("shape: {n} {w} is not {lo} to {hi}"));
        }
        let span = self.caption.shimmer_span;
        if !(span.is_finite() && span > 0.0) {
            return Err(format!(
                "shape: caption.shimmer_span {span} must be above 0"
            ));
        }
        let shares = [
            ("bar.radius_share", self.bar.radius_share),
            ("legend.corner_scale", self.legend.corner_scale[0]),
            ("legend.corner_scale", self.legend.corner_scale[1]),
            ("lang.side", self.lang.side[1]),
            ("lang.back_share", self.lang.back_share),
            ("dpad.icon_pad", self.dpad.icon_pad),
            ("dpad.hub", self.dpad.hub),
            ("dpad.stop", self.dpad.stop),
            ("press.scale", self.press.scale),
        ];
        match shares.iter().find(|(_, v)| !(0.0..=1.0).contains(v)) {
            Some((n, v)) => Err(format!("shape: {n} {v} is not 0 to 1")),
            None => Ok(()),
        }
    }
}

/// Loads and checks `shape.toml` from beside the exe or the spike folder.
pub fn load() -> Result<Shape, String> {
    read_settings(FILE, parse)
}

/// The checked sizes in `text`.
pub fn parse(text: &str) -> Result<Shape, String> {
    let s: Shape = toml::from_str(text).map_err(|e| e.to_string())?;
    s.check()?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text() -> String {
        std::fs::read_to_string(crate::config::settings_path(FILE)).expect("shape.toml reads")
    }

    #[test]
    fn the_shared_sizes_load() {
        let s = load().expect("shape.toml loads");
        assert_eq!(s.bar.button_px, [24.0, 22.0], "mock-up .tb");
        assert!(s.tip.font_px > 0.0 && s.tip.gap_px > 0.0, "tooltips");
        assert_eq!(s.line.px, 1.0, "mock-up 1px pop-line borders");
    }

    #[test]
    fn the_extra_windows_take_the_mock_up_sizes() {
        let s = load().expect("shape.toml loads");
        assert_eq!(s.pill.button_side_px, 11.0, ".selpill button");
        assert_eq!(s.caption.gap_px, 6.0, ".capbar .st gap");
        assert_eq!(s.caption.bar_px, 38.0, ".capbar with its 26 px wave");
        assert_eq!(s.snip.bar_radius_px, 9.0, ".snip-bar");
        assert_eq!(s.caption.shimmer_px, [10.0, 6.0], ".shimmer");
        assert_eq!(s.caption.shimmer_span, 2.0, ".shimmer background-size 200%");
        let heavy = text().replacen("status_weight = 600", "status_weight = 950", 1);
        assert!(
            parse(&heavy)
                .expect_err("weight")
                .contains("caption.status_weight")
        );
    }

    #[test]
    fn a_misspelt_size_is_refused() {
        let e = parse(&text().replacen("hub = 0.34", "hob = 0.34", 1)).expect_err("a typo");
        assert!(e.contains("hob"), "{e}");
    }

    #[test]
    fn an_odd_weight_or_share_is_refused() {
        let heavy = text().replacen("lit_weight = 600", "lit_weight = 950", 1);
        assert!(parse(&heavy).expect_err("weight").contains("950"));
        let scale = text().replacen("scale = 0.97", "scale = 1.5", 1);
        assert!(parse(&scale).expect_err("share").contains("press.scale"));
        let side = text().replacen("side = [9.0, 0.45]", "side = [9.0, -0.1]", 1);
        assert!(parse(&side).expect_err("share").contains("lang.side"));
        let flat = text().replacen("shimmer_span = 2.0", "shimmer_span = 0.0", 1);
        assert!(parse(&flat).expect_err("span").contains("shimmer_span"));
    }
}
