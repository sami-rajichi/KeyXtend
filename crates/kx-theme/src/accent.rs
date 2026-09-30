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

/// The first of the theme's `ink`, its `legend`, then black or white, that reaches `min` on `fill`, else the one that
/// reads best; so an ink always reaches WCAG AA, and a higher `min` is best effort.
fn ink_for(ink: Rgba, legend: Rgba, fill: Rgba, min: f32) -> Rgba {
    let inks = [ink, legend, readable(Rgba::BLACK, Rgba::WHITE, fill)];
    inks.into_iter()
        .find(|&i| contrast(i, fill) >= min)
        .unwrap_or_else(|| inks.into_iter().fold(ink, |a, b| readable(a, b, fill)))
}

/// `p` with the accent `fill` on Enter, on, locked, the LED, the ring and the badge, each ink kept readable on it.
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
        *ink = ink_for(*ink, p.legend, fill, min);
    }
    a
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Themes;
    use crate::contrast::{MAX_CONTRAST, WCAG_AA};

    /// The grid step per channel, so the sweep crosses every hue at several strengths.
    const STEP: usize = 17;
    /// Mid-tone Windows accents that once left a legend under AA (review of Task 14).
    const MID_TONES: [&str; 3] = ["#008A73", "#E74856", "#3D8F8F"];

    /// Every grey, a grid of opaque colours and the mid-tone accents.
    fn fills() -> Vec<Rgba> {
        let opaque = |r, g, b| Rgba {
            r,
            g,
            b,
            a: u8::MAX,
        };
        let levels: Vec<u8> = (0..=u8::MAX).step_by(STEP).collect();
        let mut all: Vec<Rgba> = (0..=u8::MAX).map(|v| opaque(v, v, v)).collect();
        for &r in &levels {
            for &g in &levels {
                all.extend(levels.iter().map(|&b| opaque(r, g, b)));
            }
        }
        all.extend(MID_TONES.map(|c| Rgba::parse(c).expect("a colour")));
        all
    }

    /// The four inks the accent can put on its fill.
    fn inks(p: &Palette) -> [Rgba; 4] {
        [p.enter_ink, p.on_ink, p.lock_ink, p.badge_ink]
    }

    #[test]
    fn every_accent_keeps_its_legends_at_wcag_aa_in_every_theme_and_mode() {
        let t = Themes::shipped().expect("tokens load");
        for p in t.list.iter().flat_map(|t| [&t.light, &t.dark]) {
            for fill in fills() {
                for ink in inks(&with_accent(p, fill, WCAG_AA)) {
                    let c = contrast(ink, fill);
                    assert!(c >= WCAG_AA, "{fill:?}: {ink:?} is {c}:1");
                }
            }
        }
    }

    #[test]
    fn a_minimum_no_ink_can_reach_takes_the_one_that_reads_best() {
        let native = &Themes::shipped().expect("tokens load").list[0].light;
        for fill in fills() {
            let best = contrast(Rgba::BLACK, fill).max(contrast(Rgba::WHITE, fill));
            for ink in inks(&with_accent(native, fill, MAX_CONTRAST)) {
                assert!((contrast(ink, fill) - best).abs() < 1e-3, "{fill:?}");
            }
        }
    }
}
