//! One theme's colours in one mode: shadows, skirts and the palette.

use serde::{Deserialize, Serialize};

use crate::rgba::Rgba;

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
    /// A dimmed key: the side keys' skirt faded over the plate, set from `shape.toml [dim]`.
    #[serde(skip_deserializing, default = "clear")]
    pub dim: Rgba,
}

/// The default of colours set after loading, never read from a file.
fn clear() -> Rgba {
    Rgba::CLEAR
}

/// The plate text token, and the token whose value it takes when a mode table leaves it out.
const PLATE_TEXT: (&str, &str) = ("legend_2_plate", "legend_2");

/// The press shade token, which a mode table writes only where it differs from `[common] press`.
const PRESS: &str = "press";

/// True when mode table `t` writes its own press shade.
pub(crate) fn own_press(t: &toml::Table) -> bool {
    t.contains_key(PRESS)
}

/// Gives mode table `t` its plate text from `legend_2` when the file does not write one.
pub(crate) fn plate_default(t: &mut toml::Table) {
    let (plate, from) = PLATE_TEXT;
    if !t.contains_key(plate)
        && let Some(v) = t.get(from).cloned()
    {
        t.insert(plate.into(), v);
    }
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
    /// Secondary text and icons on the plate (top bar, corner); a palette that leaves it out takes `legend_2`.
    pub legend_2_plate: Rgba,
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
    /// Laid over a pressed key, so it reads unlike the hover; a palette that leaves it out takes `[common] press`.
    #[serde(default = "clear")]
    pub press: Rgba,
    /// Every legend and icon on a pressed key; only high contrast sets one, so a theme's legends keep their colours.
    #[serde(skip_deserializing, default)]
    pub press_ink: Option<Rgba>,
    /// A key that comes in a later version: the side key faded over the plate, set from `shape.toml [dim]`.
    #[serde(skip_deserializing, default = "clear")]
    pub dim: Rgba,
    /// Legend on a dimmed key, faded the same way.
    #[serde(skip_deserializing, default = "clear")]
    pub dim_ink: Rgba,
    /// Layers under each key.
    pub key_shadow: Vec<Shadow>,
    /// Layers under the keyboard and panels.
    pub window_shadow: Vec<Shadow>,
    /// Skirt colours; only sculpted caps have them.
    #[serde(default)]
    pub skirt: Option<Skirt>,
}

impl Skirt {
    /// The skirt under the key colour called `name`, such as `key_mod`, if one has its own.
    pub(crate) fn get(&self, name: &str) -> Option<Rgba> {
        let all = [
            ("key", self.key),
            ("key_mod", self.key_mod),
            ("enter", self.enter),
            ("danger", self.danger),
            ("on", self.on),
            ("rec", self.rec),
        ];
        all.into_iter().find_map(|(n, c)| (n == name).then_some(c))
    }
}

impl Palette {
    /// Every colour token with its name in `themes.toml`.
    #[must_use]
    pub fn named(&self) -> [(&'static str, Rgba); 35] {
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
            ("legend_2_plate", self.legend_2_plate),
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
            (PRESS, self.press),
            ("dim", self.dim),
            ("dim_ink", self.dim_ink),
        ]
    }

    /// Sets the dimmed-key colours: the side key, its legend and its skirt at `share` opacity over the plate.
    pub(crate) fn fade(&mut self, share: f32) {
        self.dim = self.key_act.over(self.plate, share);
        self.dim_ink = self.legend_act.over(self.plate, share);
        if let Some(s) = &mut self.skirt {
            s.dim = s.key_mod.over(self.plate, share);
        }
    }

    /// The colour token called `name`, if there is one.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Rgba> {
        self.named()
            .into_iter()
            .find_map(|(n, c)| (n == name).then_some(c))
    }
}
