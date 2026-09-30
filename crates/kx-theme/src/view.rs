//! The look a face draws now: the chosen theme in the right mode, with the system accent or high contrast applied.

use kx_platform::ui::SysLook;
use serde::Serialize;

use crate::Themes;
use crate::accent::{accent_fill, with_accent};
use crate::common::Common;
use crate::contrast::WCAG_AA;
use crate::hc::{contrast_common, contrast_palette, hover_line};
use crate::icons::Icons;
use crate::look::{Dpad, Look};
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
    /// Shape and type of the theme.
    pub look: &'a Look,
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

/// The palette of `theme` in `mode`: system colours with `edge_px` key edges in high contrast, else the theme's,
/// with the accent if it follows it.
fn palette(theme: &Theme, mode: Mode, choice: &LookChoice, sys: &SysLook, edge_px: f32) -> Palette {
    let base = theme.palette(mode);
    if let Some(c) = &sys.contrast {
        return contrast_palette(base, c, edge_px);
    }
    let follows = choice.follow_accent && theme.look.accent;
    match sys.accent.filter(|_| follows) {
        // `max` also turns a NaN setting into WCAG AA.
        Some(a) => with_accent(
            base,
            accent_fill(&a, mode),
            choice.min_contrast.max(WCAG_AA),
        ),
        None => base.clone(),
    }
}

/// The chosen theme as it should look while the system looks like `sys`; `None` for a theme that does not exist.
#[must_use]
pub fn look_view<'a>(t: &'a Themes, choice: &LookChoice, sys: &SysLook) -> Option<LookView<'a>> {
    let theme = t.find(&choice.theme)?;
    let mode = mode_for(choice.mode, sys);
    let palette = palette(theme, mode, choice, sys, t.shape.line.px);
    let dpad = theme.look.dpad.resolve(&palette).ok()?;
    let hc = sys.contrast.as_ref();
    let common = hc.map_or(t.common, |c| contrast_common(c, &t.common));
    let sel_ink = hc.map_or(palette.pop_ink, |c| c.highlight_text.into());
    let hover_line = hc.map_or(Rgba::CLEAR, hover_line);
    Some(LookView {
        margin: reach(&palette.window_shadow),
        sel_ink,
        hover_line,
        palette,
        common,
        look: &theme.look,
        shape: &t.shape,
        motion: t.motion.pick(choice.reduced_motion || sys.anim_off),
        icons: &t.icons,
        dpad,
        dark: mode == Mode::Dark,
        contrast: hc.is_some(),
    })
}

#[cfg(test)]
mod tests;
