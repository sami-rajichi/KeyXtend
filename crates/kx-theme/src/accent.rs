//! The system accent on a theme: which shade fits the mode, the keys it fills, and whether the view keeps it.

use kx_platform::ui::Accent;

use crate::contrast::{contrast, readable};
use crate::drawn::low;
use crate::mode::Mode;
use crate::palette::Palette;
use crate::rgba::Rgba;
use crate::view::LookView;

/// The accent fill for `mode`: the darker shade on light, the lighter one on dark (Fluent).
#[must_use]
pub fn accent_fill(accent: &Accent, mode: Mode) -> Rgba {
    match mode {
        Mode::Light => accent.on_light.into(),
        Mode::Dark => accent.on_dark.into(),
    }
}

/// The first of the theme's `ink`, its `legend`, then black or white, that reaches `min` on `fill`, else the one that
/// reads best.
fn ink_for(ink: Rgba, legend: Rgba, fill: Rgba, min: f32) -> Rgba {
    let inks = [ink, legend, readable(Rgba::BLACK, Rgba::WHITE, fill)];
    inks.into_iter()
        .find(|&i| contrast(i, fill) >= min)
        .unwrap_or_else(|| inks.into_iter().fold(ink, |a, b| readable(a, b, fill)))
}

/// View `v`'s palette with the accent `fill` (see `accented`), kept only when every pair it draws still reaches its
/// minimum (`drawn::low`); else the theme's own palette.
#[must_use]
pub fn with_accent(v: &LookView, fill: Rgba, min: f32) -> Palette {
    let palette = accented(&v.palette, fill, min);
    let Ok(dpad) = v.look.dpad.resolve(&palette) else {
        return v.palette.clone();
    };
    let trial = LookView {
        palette,
        dpad,
        ..v.clone()
    };
    if low(&trial).is_empty() {
        trial.palette
    } else {
        v.palette.clone()
    }
}

/// `p` with the accent `fill` on Enter, on, locked, the LED, the ring and the badge. Each ink reaches `min` when any ink
/// can (black or white reaches 4.58:1), so it reaches WCAG AA whenever `min` is at least AA.
fn accented(p: &Palette, fill: Rgba, min: f32) -> Palette {
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
    use kx_platform::ui::SysLook;

    use super::*;
    use crate::Themes;
    use crate::contrast::{MAX_CONTRAST, WCAG_AA};
    use crate::mode::ModeChoice;
    use crate::view::{LookChoice, look_view};

    /// The grid step per channel, so the sweep crosses every hue at several strengths.
    const STEP: usize = 17;
    /// Every how many fills the drawn check samples, since it measures every pair the keyboard draws.
    const SAMPLE: usize = 5;
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
                for ink in inks(&accented(p, fill, WCAG_AA)) {
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
            for ink in inks(&accented(native, fill, MAX_CONTRAST)) {
                assert!((contrast(ink, fill) - best).abs() < 1e-3, "{fill:?}");
            }
        }
    }

    /// Native in `mode` as drawn, with its own colours.
    fn native(t: &Themes, mode: ModeChoice) -> LookView<'_> {
        let choice = LookChoice {
            theme: "native".into(),
            mode,
            follow_accent: false,
            reduced_motion: false,
            min_contrast: WCAG_AA,
        };
        look_view(t, &choice, &SysLook::default()).expect("native")
    }

    #[test]
    fn an_accent_stays_only_while_every_drawn_pair_reaches_its_minimum() {
        let t = Themes::shipped().expect("tokens load");
        let (mut kept, mut back) = (0, 0);
        for mode in [ModeChoice::Light, ModeChoice::Dark] {
            let v = native(&t, mode);
            for fill in fills().into_iter().step_by(SAMPLE) {
                let p = with_accent(&v, fill, WCAG_AA);
                if p == v.palette {
                    back += 1;
                    continue;
                }
                kept += 1;
                let dpad = v.look.dpad.resolve(&p).expect("the D-pad's tokens");
                let shown = LookView {
                    palette: p,
                    dpad,
                    ..v.clone()
                };
                assert!(low(&shown).is_empty(), "{fill:?}: {:?}", low(&shown));
            }
        }
        assert!(kept > 0 && back > 0, "kept {kept}, fell back {back}");
    }

    #[test]
    fn a_pale_accent_falls_back_to_the_themes_own_colours_and_a_dark_one_fills_the_keys() {
        let t = Themes::shipped().expect("tokens load");
        let v = native(&t, ModeChoice::Light);
        let yellow = Rgba::parse("#FFD800").expect("yellow");
        assert_eq!(
            with_accent(&v, yellow, WCAG_AA),
            v.palette,
            "unreadable on the plate"
        );
        let switched = accented(&v.palette, yellow, WCAG_AA);
        assert!(
            contrast(switched.enter_ink, yellow) >= WCAG_AA,
            "inks switch"
        );
        let teal = Rgba::parse("#1D6978").expect("teal");
        let p = with_accent(&v, teal, WCAG_AA);
        let fills = [p.enter, p.on, p.lock, p.led_on, p.ring, p.badge_bg];
        assert!(fills.iter().all(|&c| c == teal), "{p:?}");
    }
}
