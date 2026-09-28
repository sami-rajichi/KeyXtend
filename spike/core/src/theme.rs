//! The keyboard themes from `themes.toml`: three looks, each light and soft dark.

mod common;
mod extras;
mod look;
mod palette;
mod shape;

use serde::Deserialize;

use crate::config::settings_path;
pub use common::Common;
pub use look::{Cap, Dpad, DpadRefs, Gradient, Look};
pub use palette::{Palette, Rgba, Shadow, Skirt, contrast};
pub use shape::Shape;

/// The themes file, beside `spike.toml`.
const FILE: &str = "themes.toml";
/// Text weights run from thin to black.
const FONT_WEIGHTS: std::ops::RangeInclusive<u16> = 100..=900;

/// Light or soft dark.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The light palette.
    Light,
    /// The soft dark palette.
    Dark,
}

impl Mode {
    /// Both modes, light first.
    pub const ALL: [Mode; 2] = [Mode::Light, Mode::Dark];

    /// The table name of this mode in `themes.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }
}

/// One theme: its look and its two palettes.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    /// Shape and type.
    pub look: Look,
    /// Light colours.
    pub light: Palette,
    /// Soft dark colours.
    pub dark: Palette,
}

impl Theme {
    /// The palette for `mode`.
    pub fn palette(&self, mode: Mode) -> &Palette {
        match mode {
            Mode::Light => &self.light,
            Mode::Dark => &self.dark,
        }
    }
}

/// The file as written: a list of theme tables and the colours they share.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ThemeFile {
    theme: Vec<toml::Table>,
    common: Common,
}

/// Every theme, in file order, and the colours and sizes they share.
#[derive(Debug, Clone, PartialEq)]
pub struct Themes {
    /// The themes, as the theme bar lists them.
    pub list: Vec<Theme>,
    /// Shared colours.
    pub common: Common,
    /// Shared sizes.
    pub shape: Shape,
}

/// Loads `themes.toml` and `shape.toml` from beside the exe or the spike folder.
pub fn load() -> Result<Themes, String> {
    let file = settings_path(FILE);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    let (list, common) = parse(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    Ok(Themes {
        list,
        common,
        shape: shape::load()?,
    })
}

/// The themes in `text`, in file order, and their shared colours; an error names the theme and mode at fault.
pub fn parse(text: &str) -> Result<(Vec<Theme>, Common), String> {
    let ThemeFile {
        theme: tables,
        common,
    } = toml::from_str(text).map_err(|e| e.to_string())?;
    let themes = tables
        .into_iter()
        .map(theme)
        .collect::<Result<Vec<_>, _>>()?;
    if themes.is_empty() {
        return Err("no [[theme]] tables".into());
    }
    for (i, t) in themes.iter().enumerate() {
        if themes[..i].iter().any(|u| u.look.id == t.look.id) {
            return Err(format!("theme id {:?} is used twice", t.look.id));
        }
    }
    Ok((themes, common))
}

/// One theme table: its two mode tables, then its look.
fn theme(mut t: toml::Table) -> Result<Theme, String> {
    let id = t
        .get("id")
        .and_then(toml::Value::as_str)
        .unwrap_or("?")
        .to_string();
    let light = palette(&id, Mode::Light, t.remove(Mode::Light.key()))?;
    let dark = palette(&id, Mode::Dark, t.remove(Mode::Dark.key()))?;
    let look: Look = toml::Value::Table(t)
        .try_into()
        .map_err(|e| format!("{id}: {e}"))?;
    for p in [&light, &dark] {
        look.check(p).map_err(|e| format!("{id}: {e}"))?;
    }
    Ok(Theme { look, light, dark })
}

/// One mode table, with errors prefixed `<id>.<mode>`.
fn palette(id: &str, mode: Mode, table: Option<toml::Value>) -> Result<Palette, String> {
    let at = format!("{id}.{}", mode.key());
    let table = table.ok_or_else(|| format!("{at}: missing"))?;
    table.try_into().map_err(|e| format!("{at}: {e}"))
}

/// Whichever of `a` and `b` reads better on `bg`.
fn readable(a: Rgba, b: Rgba, bg: Rgba) -> Rgba {
    if contrast(a, bg) >= contrast(b, bg) {
        a
    } else {
        b
    }
}

/// `p` with the accent `fill` on Enter, on, locked, the LED, the ring and the badge.
/// An ink that would drop below `min` on the fill switches to the main legend if that reads better.
pub fn with_accent(p: &Palette, fill: Rgba, min: f32) -> Palette {
    let mut a = p.clone();
    for c in [
        &mut a.enter,
        &mut a.on,
        &mut a.lock,
        &mut a.led_on,
        &mut a.ring,
        &mut a.badge_bg,
    ] {
        *c = fill;
    }
    for ink in [
        &mut a.enter_ink,
        &mut a.on_ink,
        &mut a.lock_ink,
        &mut a.badge_ink,
    ] {
        if contrast(*ink, fill) < min {
            *ink = readable(*ink, p.legend, fill);
        }
    }
    a
}

/// Every legend below `min` contrast with its background, as `<id>.<mode>: <pair> is <n>:1`.
pub fn low_contrast(themes: &[Theme], min: f32) -> Vec<String> {
    let mut low = Vec::new();
    for t in themes {
        for mode in Mode::ALL {
            for (pair, ink, bg) in t.palette(mode).legend_pairs() {
                let c = contrast(ink, bg);
                if c < min {
                    low.push(format!("{}.{}: {pair} is {c:.1}:1", t.look.id, mode.key()));
                }
            }
        }
    }
    low
}

#[cfg(test)]
mod tests;
