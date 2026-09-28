//! `spike.toml [look]`: the starting theme and mode, accent themes, the frosted plate, the tooltip delay and motion switches.

use serde::{Deserialize, Serialize};

use crate::theme::Theme;

/// Every possible WCAG contrast ratio.
const CONTRAST: std::ops::RangeInclusive<f32> = 1.0..=21.0;

/// Which palette the keyboard shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
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
    /// Contrast a legend keeps when its key takes the accent colour.
    pub min_contrast: f32,
    /// Native's frosted plate.
    pub backdrop: Backdrop,
    /// Share of the plate colour kept over a backdrop, 0 to 1.
    pub plate_alpha: f32,
    /// Turns every animation off except the hold ring.
    pub reduced_motion: bool,
    /// Themes whose Enter, on and lock colours follow the Windows accent.
    pub accent_themes: Vec<String>,
    /// Hover time before a button's tooltip shows, in ms.
    pub tip_ms: u64,
}

impl LookConfig {
    /// Refuses a plate share outside 0 to 1, a contrast outside 1:1 to 21:1 (NaN fails both) and a zero tooltip delay.
    pub fn check(&self) -> Result<(), String> {
        let share = |v: f32| (0.0..=1.0).contains(&v);
        if !share(self.plate_alpha) {
            return Err("look: plate_alpha must be 0 to 1".into());
        }
        if !CONTRAST.contains(&self.min_contrast) {
            let (lo, hi) = (CONTRAST.start(), CONTRAST.end());
            return Err(format!("look: min_contrast is a ratio from {lo} to {hi}"));
        }
        if self.tip_ms == 0 {
            return Err("look: tip_ms must be above 0".into());
        }
        Ok(())
    }

    /// Refuses a starting or accent theme that `themes` lacks, and an accent on sculpted caps, whose skirts keep their colour.
    pub fn check_themes(&self, themes: &[Theme]) -> Result<(), String> {
        let find = |id: &str| {
            let t = themes.iter().find(|t| t.look.id == id);
            t.ok_or_else(|| format!("look: themes.toml has no theme {id:?}"))
        };
        find(&self.theme)?;
        for id in &self.accent_themes {
            if find(id)?.light.skirt.is_some() {
                return Err(format!(
                    "look: {id} has sculpted caps, so it cannot follow the accent"
                ));
            }
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
        c.min_contrast = 0.5;
        assert!(c.check().is_err(), "contrast starts at 1:1");
        c.min_contrast = f32::NAN;
        assert!(c.check().is_err(), "NaN is refused");
    }

    #[test]
    fn theme_ids_must_exist_and_sculpted_caps_never_follow_the_accent() {
        let themes = crate::theme::load().expect("themes load").list;
        let mut c = cfg();
        assert_eq!(c.check_themes(&themes), Ok(()));
        c.theme = "natve".into();
        assert!(c.check_themes(&themes).expect_err("typo").contains("natve"));
        let mut c = cfg();
        c.accent_themes = vec!["natve".into()];
        assert!(c.check_themes(&themes).is_err(), "an accent typo");
        c.accent_themes = vec!["dolch".into()];
        assert!(
            c.check_themes(&themes).is_err(),
            "its skirts keep their colour"
        );
        c.accent_themes = vec!["native".into()];
        assert_eq!(c.check_themes(&themes), Ok(()));
    }

    #[test]
    fn a_zero_tooltip_delay_is_refused() {
        let mut c = cfg();
        assert!(c.tip_ms > 0);
        c.tip_ms = 0;
        assert!(
            c.check().is_err(),
            "tips would flash while passing over keys"
        );
    }
}
