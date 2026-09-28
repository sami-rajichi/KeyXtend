//! `spike.toml [look]`: the starting theme and mode, the frosted plate, fading and motion switches.

use serde::Deserialize;

/// Every possible WCAG contrast ratio.
const CONTRAST: std::ops::RangeInclusive<f32> = 1.0..=21.0;

/// Which palette the keyboard shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModeChoice {
    /// Always light.
    Light,
    /// Always soft dark.
    Dark,
    /// Follow Windows' app mode.
    Auto,
}

/// What Native Adaptive puts behind its plate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backdrop {
    /// Nothing: a solid plate.
    None,
    /// The DWM Mica backdrop.
    Mica,
    /// The DWM Acrylic backdrop.
    Acrylic,
    /// The accent-policy blur, which stays on an inactive window.
    Blur,
}

/// `spike.toml [look]`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LookConfig {
    /// Starting theme id from `themes.toml`.
    pub theme: String,
    /// Starting mode.
    pub mode: ModeChoice,
    /// Legends below this contrast are reported.
    pub min_contrast: f32,
    /// Native's frosted plate.
    pub backdrop: Backdrop,
    /// Share of the plate colour kept over a backdrop, 0 to 1.
    pub plate_alpha: f32,
    /// Fade after this long without the pointer, in ms.
    pub idle_fade_ms: u64,
    /// Opacity once faded, 0 to 1.
    pub idle_opacity: f32,
    /// Turns every animation off except the hold ring.
    pub reduced_motion: bool,
}

impl LookConfig {
    /// Refuses shares outside 0 to 1 and a contrast outside 1:1 to 21:1; NaN fails both.
    pub fn check(&self) -> Result<(), String> {
        let share = |v: f32| (0.0..=1.0).contains(&v);
        if !share(self.plate_alpha) || !share(self.idle_opacity) {
            return Err("look: plate_alpha and idle_opacity must be 0 to 1".into());
        }
        if !CONTRAST.contains(&self.min_contrast) {
            return Err("look: min_contrast is a ratio from 1 to 21".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> LookConfig {
        crate::config::load().expect("spike.toml loads").look
    }

    #[test]
    fn the_look_starts_on_native_following_windows() {
        let c = cfg();
        assert_eq!(c.theme, "native");
        assert_eq!(c.mode, ModeChoice::Auto);
        assert_eq!(c.check(), Ok(()));
    }

    #[test]
    fn shares_outside_zero_to_one_are_refused() {
        let mut c = cfg();
        c.plate_alpha = 1.5;
        assert!(c.check().is_err());
        let mut c = cfg();
        c.idle_opacity = -0.1;
        assert!(c.check().is_err());
        let mut c = cfg();
        c.min_contrast = 0.5;
        assert!(c.check().is_err(), "contrast starts at 1:1");
        c.min_contrast = f32::NAN;
        assert!(c.check().is_err(), "NaN is refused");
    }
}
