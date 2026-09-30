//! The system accent on a theme: which shade fits the mode, and the keys it fills.

use kx_platform::ui::Accent;

use crate::contrast::{contrast, readable};
use crate::mode::Mode;
use crate::palette::Palette;
use crate::rgba::Rgba;

/// The accent fill for `mode`: the darker shade on light, the lighter one on dark (Fluent).
#[must_use]
pub fn accent_fill(accent: &Accent, mode: Mode) -> Rgba {
    match mode {
        Mode::Light => accent.on_light.into(),
        Mode::Dark => accent.on_dark.into(),
    }
}

/// `p` with the accent `fill` on Enter, on, locked, the LED, the ring and the badge.
/// An ink that would drop below `min` on the fill switches to the main legend if that reads better.
#[must_use]
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
