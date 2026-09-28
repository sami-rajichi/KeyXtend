//! A theme's shape and type, shared by both modes: radii, gaps, fonts, glows, how caps are built and the D-pad's colours.

use serde::{Deserialize, Serialize};

use super::FONT_WEIGHTS;
use super::palette::{Palette, Rgba};

/// A two-way cap gradient: `lighten` mixes in white at the start, `darken` black at the end.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Gradient {
    /// Share of white at the start, 0 to 1.
    pub lighten: f32,
    /// Where the plain key colour sits, 0 to 1.
    pub mid_stop: f32,
    /// Share of black at the end, 0 to 1.
    pub darken: f32,
    /// Runs bottom to top instead of top to bottom.
    pub upward: bool,
}

impl Gradient {
    /// True when every share is 0 to 1.
    fn is_valid(&self) -> bool {
        [self.lighten, self.mid_stop, self.darken]
            .iter()
            .all(|v| (0.0..=1.0).contains(v))
    }
}

/// How a key cap is built.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Cap {
    /// One flat colour; braces so that stray fields are refused too.
    Flat {},
    /// A convex face that flips its gradient when pressed.
    Convex {
        /// At rest.
        rest: Gradient,
        /// While pressed.
        pressed: Gradient,
    },
    /// A skirt in the skirt colour with a lighter top inset into it.
    Skirt {
        /// The top's gradient.
        top: Gradient,
        /// Top inset from the cap edge: top, right, bottom, left.
        top_inset_px: [f32; 4],
        /// The same while pressed.
        pressed_inset_px: [f32; 4],
        /// Corner radius of the top.
        top_radius_px: f32,
        /// Legends move up by this much.
        legend_lift_px: f32,
    },
}

impl Cap {
    /// Its gradients, for checks.
    fn gradients(&self) -> Vec<Gradient> {
        match self {
            Cap::Flat {} => Vec::new(),
            Cap::Convex { rest, pressed } => vec![*rest, *pressed],
            Cap::Skirt { top, .. } => vec![*top],
        }
    }
}

/// The D-pad's colours, as names of palette tokens.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DpadRefs {
    /// The four arrows.
    pub face: String,
    /// Their icons.
    pub ink: String,
    /// The arrow that is scrolling.
    pub on: String,
    /// Its icon.
    pub on_ink: String,
    /// The Stop button.
    pub stop: String,
    /// Its icon.
    pub stop_ink: String,
}

/// The D-pad's colours in one palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Dpad {
    /// The four arrows.
    pub face: Rgba,
    /// Their icons.
    pub ink: Rgba,
    /// The arrow that is scrolling.
    pub on: Rgba,
    /// Its icon.
    pub on_ink: Rgba,
    /// The Stop button.
    pub stop: Rgba,
    /// Its icon.
    pub stop_ink: Rgba,
}

impl DpadRefs {
    /// The colours these names give in `p`; an error names the first unknown token.
    pub fn resolve(&self, p: &Palette) -> Result<Dpad, String> {
        let c = |n: &str| p.get(n).ok_or_else(|| format!("dpad: no token {n:?}"));
        Ok(Dpad {
            face: c(&self.face)?,
            ink: c(&self.ink)?,
            on: c(&self.on)?,
            on_ink: c(&self.on_ink)?,
            stop: c(&self.stop)?,
            stop_ink: c(&self.stop_ink)?,
        })
    }
}

/// A theme's shape and type, shared by both modes.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Look {
    /// Short id, such as `native`.
    pub id: String,
    /// Name shown in the theme bar.
    pub name: String,
    /// Key corner radius.
    pub key_radius_px: f32,
    /// Keyboard and panel corner radius.
    pub window_radius_px: f32,
    /// Gap between keys.
    pub gap_px: f32,
    /// Legend size.
    pub legend_px: f32,
    /// Legend size on modifiers.
    pub mod_legend_px: f32,
    /// Legend weight, 100 to 900.
    pub legend_weight: u16,
    /// Arabic legends are this much larger.
    pub arabic_scale: f32,
    /// Latin fonts, first found wins.
    pub latin_fonts: Vec<String>,
    /// Arabic fonts, first found wins.
    pub arabic_fonts: Vec<String>,
    /// Glow around a lit LED.
    pub led_glow_px: f32,
    /// Glow around a locked key, in the `on` colour; 0 for none.
    pub lock_glow_px: f32,
    /// Opacity of the lock glow's `on` colour, 0 to 1.
    pub lock_glow_mix: f32,
    /// How caps are built.
    pub cap: Cap,
    /// The D-pad's colours.
    pub dpad: DpadRefs,
}

impl Look {
    /// Refuses shares outside 0 to 1, an odd weight, and D-pad names that `p` lacks.
    pub fn check(&self, p: &Palette) -> Result<(), String> {
        if !FONT_WEIGHTS.contains(&self.legend_weight) {
            let (w, lo, hi) = (self.legend_weight, FONT_WEIGHTS.start(), FONT_WEIGHTS.end());
            return Err(format!("legend_weight {w} is not {lo} to {hi}"));
        }
        let share = (0.0..=1.0).contains(&self.lock_glow_mix);
        if !share || !self.cap.gradients().iter().all(Gradient::is_valid) {
            return Err("lock_glow_mix and gradient shares must be 0 to 1".into());
        }
        self.dpad.resolve(p).map(|_| ())
    }
}
