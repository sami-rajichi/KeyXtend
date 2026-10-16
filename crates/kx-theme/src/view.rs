//! The look a face draws now: the chosen theme in the right mode, with the system accent or high contrast applied.

use std::borrow::Cow;

use kx_platform::ui::SysLook;
use serde::Serialize;

use crate::Themes;
use crate::accent::{accent_fill, with_accent};
use crate::common::Common;
use crate::contrast::WCAG_AA;
use crate::hc::{contrast_common, contrast_palette, hover_line};
use crate::icons::Icons;
use crate::look::{Cap, Dpad, Look};
use crate::mode::{Mode, ModeChoice, mode_for};
use crate::motion::Motion;
use crate::palette::{Palette, Shadow};
use crate::rgba::Rgba;
use crate::shape::Shape;
use crate::theme::Theme;

/// What the user chose for the look: the `[ui]` settings the look view reads.
#[derive(Debug, Clone, PartialEq)]
pub struct LookChoice {
    /// Theme id from `themes.toml`.
    pub theme: String,
    /// Light, soft dark or following the system.
    pub mode: ModeChoice,
    /// Themes that can take the system accent take it on Enter, on and locked keys.
    pub follow_accent: bool,
    /// Turns every animation off except the hold ring.
    pub reduced_motion: bool,
    /// Contrast a legend keeps when its key takes the accent: at least WCAG AA, and a higher value is best effort.
    pub min_contrast: f32,
}

/// Everything a face needs to paint one frame of the look.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LookView<'a> {
    /// Colours, after the accent or high contrast.
    pub palette: Palette,
    /// Colours every theme shares, as system colours in high contrast.
    pub common: Common,
    /// Shape and type of the theme; caps are flat in high contrast, as effects are off.
    pub look: Cow<'a, Look>,
    /// Sizes all themes share.
    pub shape: &'a Shape,
    /// Animations; every length is 0 with reduced motion.
    pub motion: &'a Motion,
    /// The icon each part shows.
    pub icons: &'a Icons,
    /// The D-pad's colours.
    pub dpad: Dpad,
    /// The mode shown.
    pub dark: bool,
    /// High contrast is on, so effects and glows are off.
    pub contrast: bool,
    /// Text on `pop_sel` and `pop_hover`: `pop_ink`, or `HighlightText` in high contrast, where both fills are `Highlight`.
    pub sel_ink: Rgba,
    /// The outline of a key under the pointer, `shape.hc_outline.px` wide: `Highlight` in high contrast, else clear.
    pub hover_line: Rgba,
    /// Room the window shadow needs around the plate, at size 1.
    pub margin: f32,
    /// Room the badge's shadow needs around the mode badge.
    pub badge_margin: f32,
}

/// How far `layers` reach past the shape they sit under.
#[must_use]
pub fn reach(layers: &[Shadow]) -> f32 {
    let far = |s: &Shadow| s.x.abs().max(s.y.abs()) + s.blur + s.spread;
    layers
        .iter()
        .filter(|s| !s.inset)
        .map(far)
        .fold(0.0, f32::max)
        .ceil()
}

/// The palette of `theme` in `mode`: system colours with `edge_px` key edges in high contrast, else the theme's own.
fn palette(theme: &Theme, mode: Mode, sys: &SysLook, edge_px: f32) -> Palette {
    let base = theme.palette(mode);
    match &sys.contrast {
        Some(c) => contrast_palette(base, c, edge_px),
        None => base.clone(),
    }
}

/// `look` with flat caps: a cap's gradient tints are not system colours.
fn flat(look: &Look) -> Look {
    Look {
        cap: Cap::Flat {},
        ..look.clone()
    }
}

/// `row_px` when it is a finite row above 0, which the drawn check can measure caps by.
fn measurable(row_px: Option<f32>) -> Option<f32> {
    row_px.filter(|r| r.is_finite() && *r > 0.0)
}

/// The chosen theme as it should look while the system looks like `sys`; `None` for a theme that does not exist.
/// The accent is checked on the keyboard's rows, `row_px` high at size 1; without a finite row above 0 it is not tried.
#[must_use]
pub fn look_view<'a>(
    t: &'a Themes,
    choice: &LookChoice,
    sys: &SysLook,
    row_px: Option<f32>,
) -> Option<LookView<'a>> {
    let theme = t.find(&choice.theme)?;
    let mode = mode_for(choice.mode, sys);
    let palette = palette(theme, mode, sys, t.shape.line.px);
    let dpad = theme.look.dpad.resolve(&palette).ok()?;
    let hc = sys.contrast.as_ref();
    let common = hc.map_or(t.common, |c| contrast_common(c, &t.common));
    let sel_ink = hc.map_or(palette.pop_ink, |c| c.highlight_text.into());
    let hover_line = hc.map_or(Rgba::CLEAR, hover_line);
    let mut view = LookView {
        margin: reach(&palette.window_shadow),
        badge_margin: reach(&palette.badge_shadow),
        sel_ink,
        hover_line,
        palette,
        common,
        look: hc.map_or(Cow::Borrowed(&theme.look), |_| {
            Cow::Owned(flat(&theme.look))
        }),
        shape: &t.shape,
        motion: t.motion.pick(choice.reduced_motion || sys.anim_off),
        icons: &t.icons,
        dpad,
        dark: mode == Mode::Dark,
        contrast: hc.is_some(),
    };
    // High contrast wins over the accent; `max` also turns a NaN setting into WCAG AA.
    let follows = choice.follow_accent && theme.look.accent && hc.is_none();
    if let (Some(a), Some(row)) = (sys.accent.filter(|_| follows), measurable(row_px)) {
        let min = choice.min_contrast.max(WCAG_AA);
        view.palette = with_accent(&view, accent_fill(&a, mode), min, row);
        view.dpad = theme.look.dpad.resolve(&view.palette).ok()?;
    }
    Some(view)
}

#[cfg(test)]
mod tests;
