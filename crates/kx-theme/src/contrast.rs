//! WCAG 2.x contrast: the ratio between two colours, the standard's minimums, and the legends below one.

use std::ops::RangeInclusive;

use crate::mode::Mode;
use crate::rgba::Rgba;
use crate::theme::Theme;

/// WCAG 2.x level AA's least contrast for text (success criterion 1.4.3).
pub const WCAG_AA: f32 = 4.5;
/// WCAG 2.x's least contrast for shapes that carry meaning, such as the hold ring (success criterion 1.4.11).
pub const WCAG_SHAPES: f32 = 3.0;
/// The highest contrast there is: black on white.
pub const MAX_CONTRAST: f32 = 21.0;
/// The contrasts a user may ask legends to keep: from WCAG AA up, so `[ui] min_contrast` can never go below it.
pub const MIN_CONTRAST: RangeInclusive<f32> = WCAG_AA..=MAX_CONTRAST;

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
#[must_use]
pub fn contrast(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + FLARE) / (la.min(lb) + FLARE)
}

/// Whichever of `a` and `b` reads better on `bg`.
#[must_use]
pub fn readable(a: Rgba, b: Rgba, bg: Rgba) -> Rgba {
    if contrast(a, bg) >= contrast(b, bg) {
        a
    } else {
        b
    }
}

/// Every legend below `min` contrast with its background, as `<id>.<mode>: <pair> is <n>:1`.
#[must_use]
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
