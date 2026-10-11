//! `themes.toml`: three looks, each light and soft dark, and the colours they share.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::common::Common;
use crate::look::Look;
use crate::mode::Mode;
use crate::palette::{Palette, own_press, plate_default};
use crate::rgba::Rgba;

/// Text weights run from thin to black.
pub(crate) const FONT_WEIGHTS: std::ops::RangeInclusive<u16> = 100..=900;

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
    #[must_use]
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

/// The first of `values` outside 0 to 1, NaN included, with its name.
pub(crate) fn outside_unit<'a>(values: &[(&'a str, f32)]) -> Option<(&'a str, f32)> {
    values
        .iter()
        .copied()
        .find(|(_, v)| !(0.0..=1.0).contains(v))
}

/// Curve `name` from `curves` as Qt bezier points, ending at (1, 1) as Qt wants.
pub(crate) fn bez(curves: &BTreeMap<String, [f32; 4]>, name: &str) -> Option<[f32; 6]> {
    let [x1, y1, x2, y2] = *curves.get(name)?;
    Some([x1, y1, x2, y2, 1.0, 1.0])
}

/// The themes in `text`, in file order, and their shared colours; an error names the theme and mode at fault.
pub(crate) fn parse(text: &str) -> Result<(Vec<Theme>, Common), String> {
    let ThemeFile {
        theme: tables,
        common,
    } = toml::from_str(text).map_err(|e| e.to_string())?;
    let themes = tables
        .into_iter()
        .map(|t| theme(t, common.press))
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

/// One theme table: its two mode tables, pressed with the shared `press` unless they write their own, then its look,
/// whose marks take its Arabic fonts unless it names its own.
fn theme(mut t: toml::Table, press: Rgba) -> Result<Theme, String> {
    let id = t
        .get("id")
        .and_then(toml::Value::as_str)
        .unwrap_or("?")
        .to_string();
    let light = palette(&id, Mode::Light, t.remove(Mode::Light.key()), press)?;
    let dark = palette(&id, Mode::Dark, t.remove(Mode::Dark.key()), press)?;
    let mut look: Look = toml::Value::Table(t)
        .try_into()
        .map_err(|e| format!("{id}: {e}"))?;
    if look.mark_fonts.is_empty() {
        look.mark_fonts.clone_from(&look.arabic_fonts);
    }
    for p in [&light, &dark] {
        look.check(p).map_err(|e| format!("{id}: {e}"))?;
    }
    Ok(Theme { look, light, dark })
}

/// One mode table, with its plate text and press shade defaulted and errors prefixed `<id>.<mode>`. Unless it writes
/// its own, a light table lays the shared `press` over its darkening hover, so a press reads darker still.
fn palette(
    id: &str,
    mode: Mode,
    table: Option<toml::Value>,
    press: Rgba,
) -> Result<Palette, String> {
    let at = format!("{id}.{}", mode.key());
    let mut table = table.ok_or_else(|| format!("{at}: missing"))?;
    let own = table.as_table_mut().is_some_and(|t| {
        plate_default(t);
        own_press(t)
    });
    let mut p: Palette = table.try_into().map_err(|e| format!("{at}: {e}"))?;
    if !own {
        p.press = match mode {
            Mode::Light => press.stack(p.hover),
            Mode::Dark => press,
        };
    }
    Ok(p)
}

#[cfg(test)]
mod press;
#[cfg(test)]
mod tests;
