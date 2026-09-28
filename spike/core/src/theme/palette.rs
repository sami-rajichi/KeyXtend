//! One theme's colours in one mode, and the WCAG contrast between two colours.

use serde::{Deserialize, Serialize, Serializer};

/// Hex digits in `#RRGGBB`.
const RGB_DIGITS: usize = 6;
/// Hex digits in `#RRGGBBAA`.
const RGBA_DIGITS: usize = 8;
/// sRGB channels at or below this are linear (WCAG 2.x).
const LINEAR_LIMIT: f32 = 0.040_45;
/// Divisor for the linear part of the sRGB curve.
const LINEAR_DIV: f32 = 12.92;
/// Offset of the sRGB curve.
const CURVE_OFFSET: f32 = 0.055;
/// Exponent of the sRGB curve.
const CURVE_EXP: f32 = 2.4;
/// Red, green and blue weights of relative luminance.
const LUMA_WEIGHTS: [f32; 3] = [0.2126, 0.7152, 0.0722];
/// Flare added to both luminances in the contrast ratio.
const FLARE: f32 = 0.05;

/// A colour with alpha, written `#RRGGBB` or `#RRGGBBAA`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha; 255 is opaque.
    pub a: u8,
}

impl Rgba {
    /// Reads `#RRGGBB` (opaque) or `#RRGGBBAA`.
    pub fn parse(s: &str) -> Result<Rgba, String> {
        let bad = || format!("{s:?} is not #RRGGBB or #RRGGBBAA");
        let hex = s.strip_prefix('#').ok_or_else(bad)?;
        let digits = hex.bytes().all(|b| b.is_ascii_hexdigit());
        if !digits || !matches!(hex.len(), RGB_DIGITS | RGBA_DIGITS) {
            return Err(bad());
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| bad());
        let a = if hex.len() == RGBA_DIGITS {
            byte(6)?
        } else {
            u8::MAX
        };
        Ok(Rgba {
            r: byte(0)?,
            g: byte(2)?,
            b: byte(4)?,
            a,
        })
    }
}

impl Serialize for Rgba {
    /// Written as Qt reads colours: `#AARRGGBB`.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            self.a, self.r, self.g, self.b
        ))
    }
}

impl TryFrom<String> for Rgba {
    type Error = String;

    fn try_from(s: String) -> Result<Self, String> {
        Rgba::parse(&s)
    }
}

/// One channel as linear light.
fn linear(c: u8) -> f32 {
    let s = f32::from(c) / f32::from(u8::MAX);
    if s <= LINEAR_LIMIT {
        s / LINEAR_DIV
    } else {
        ((s + CURVE_OFFSET) / (1.0 + CURVE_OFFSET)).powf(CURVE_EXP)
    }
}

/// Relative luminance, ignoring alpha.
fn luminance(c: Rgba) -> f32 {
    let [r, g, b] = LUMA_WEIGHTS;
    r * linear(c.r) + g * linear(c.g) + b * linear(c.b)
}

/// The WCAG contrast ratio between two colours, from 1 to 21.
pub fn contrast(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + FLARE) / (la.min(lb) + FLARE)
}

/// One shadow layer, as in CSS `box-shadow`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Shadow {
    /// Drawn inside the shape.
    #[serde(default)]
    pub inset: bool,
    /// Offset right, in logical pixels.
    pub x: f32,
    /// Offset down, in logical pixels.
    pub y: f32,
    /// Blur radius.
    pub blur: f32,
    /// Growth before the blur.
    pub spread: f32,
    /// Colour.
    pub color: Rgba,
}

/// The skirt colours of a sculpted cap, by key type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Skirt {
    /// Letter keys.
    pub key: Rgba,
    /// Modifiers, the language key and side keys.
    pub key_mod: Rgba,
    /// Enter.
    pub enter: Rgba,
    /// Esc and Backspace.
    pub danger: Rgba,
    /// Keys that are on or locked.
    pub on: Rgba,
    /// The recording key.
    pub rec: Rgba,
}

/// Every colour of one theme in one mode; the names follow the mock-up's CSS tokens.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    /// The plate behind the keys.
    pub plate: Rgba,
    /// Letter keys.
    pub key: Rgba,
    /// Modifiers and the language key.
    pub key_mod: Rgba,
    /// Side keys.
    pub key_act: Rgba,
    /// Enter.
    pub enter: Rgba,
    /// Legend on Enter.
    pub enter_ink: Rgba,
    /// Esc and Backspace.
    pub danger: Rgba,
    /// Legend on Esc and Backspace.
    pub danger_ink: Rgba,
    /// Main legend.
    pub legend: Rgba,
    /// Legend on modifiers.
    pub legend_mod: Rgba,
    /// Second and third legends.
    pub legend_2: Rgba,
    /// Legend on side keys.
    pub legend_act: Rgba,
    /// A key that is on.
    pub on: Rgba,
    /// Legend on a key that is on.
    pub on_ink: Rgba,
    /// A locked key.
    pub lock: Rgba,
    /// Legend on a locked key.
    pub lock_ink: Rgba,
    /// The recording key.
    pub rec: Rgba,
    /// Legend on the recording key.
    pub rec_ink: Rgba,
    /// LED when off.
    pub led_off: Rgba,
    /// LED when on.
    pub led_on: Rgba,
    /// The hold ring.
    pub ring: Rgba,
    /// Mode badge background.
    pub badge_bg: Rgba,
    /// Mode badge text.
    pub badge_ink: Rgba,
    /// Panel background.
    pub pop_bg: Rgba,
    /// Panel text.
    pub pop_ink: Rgba,
    /// Panel secondary text.
    pub pop_muted: Rgba,
    /// Panel dividers.
    pub pop_line: Rgba,
    /// Panel row under the pointer.
    pub pop_hover: Rgba,
    /// Selected panel row.
    pub pop_sel: Rgba,
    /// Suggestion chips.
    pub chip_bg: Rgba,
    /// Laid over a key under the pointer: darker on light, lighter on dark.
    pub hover: Rgba,
    /// Laid over a pressed key (mock-up `brightness(.94)`).
    pub press: Rgba,
    /// Layers under each key.
    pub key_shadow: Vec<Shadow>,
    /// Layers under the keyboard and panels.
    pub window_shadow: Vec<Shadow>,
    /// Skirt colours; only sculpted caps have them.
    #[serde(default)]
    pub skirt: Option<Skirt>,
}

impl Palette {
    /// Every colour token with its name in `themes.toml`.
    pub fn named(&self) -> [(&'static str, Rgba); 32] {
        [
            ("plate", self.plate),
            ("key", self.key),
            ("key_mod", self.key_mod),
            ("key_act", self.key_act),
            ("enter", self.enter),
            ("enter_ink", self.enter_ink),
            ("danger", self.danger),
            ("danger_ink", self.danger_ink),
            ("legend", self.legend),
            ("legend_mod", self.legend_mod),
            ("legend_2", self.legend_2),
            ("legend_act", self.legend_act),
            ("on", self.on),
            ("on_ink", self.on_ink),
            ("lock", self.lock),
            ("lock_ink", self.lock_ink),
            ("rec", self.rec),
            ("rec_ink", self.rec_ink),
            ("led_off", self.led_off),
            ("led_on", self.led_on),
            ("ring", self.ring),
            ("badge_bg", self.badge_bg),
            ("badge_ink", self.badge_ink),
            ("pop_bg", self.pop_bg),
            ("pop_ink", self.pop_ink),
            ("pop_muted", self.pop_muted),
            ("pop_line", self.pop_line),
            ("pop_hover", self.pop_hover),
            ("pop_sel", self.pop_sel),
            ("chip_bg", self.chip_bg),
            ("hover", self.hover),
            ("press", self.press),
        ]
    }

    /// The colour token called `name`, if there is one.
    pub fn get(&self, name: &str) -> Option<Rgba> {
        self.named()
            .into_iter()
            .find_map(|(n, c)| (n == name).then_some(c))
    }

    /// Each text colour with the background it sits on, named for reports.
    pub fn legend_pairs(&self) -> [(&'static str, Rgba, Rgba); 13] {
        [
            ("legend on key", self.legend, self.key),
            ("legend_2 on key", self.legend_2, self.key),
            ("legend_2 on plate", self.legend_2, self.plate),
            ("legend_mod on key_mod", self.legend_mod, self.key_mod),
            ("legend_act on key_act", self.legend_act, self.key_act),
            ("enter_ink on enter", self.enter_ink, self.enter),
            ("danger_ink on danger", self.danger_ink, self.danger),
            ("on_ink on on", self.on_ink, self.on),
            ("lock_ink on lock", self.lock_ink, self.lock),
            ("rec_ink on rec", self.rec_ink, self.rec),
            ("pop_ink on pop_bg", self.pop_ink, self.pop_bg),
            ("pop_muted on pop_bg", self.pop_muted, self.pop_bg),
            ("badge_ink on badge_bg", self.badge_ink, self.badge_bg),
        ]
    }
}
