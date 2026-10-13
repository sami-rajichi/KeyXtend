//! Light or soft dark: the mode a user picks, and the mode shown once the system's own is known.

use kx_platform::ui::SysLook;
use serde::{Deserialize, Serialize};

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
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
        }
    }
}

/// Which palette the user wants, as the `[ui] mode` setting writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModeChoice {
    /// Always light.
    Light,
    /// Always soft dark.
    Dark,
    /// Follow the system's app mode.
    Auto,
}

/// The mode to show for `choice`, given the system look.
#[must_use]
pub fn mode_for(choice: ModeChoice, look: &SysLook) -> Mode {
    let dark = match choice {
        ModeChoice::Light => false,
        ModeChoice::Dark => true,
        ModeChoice::Auto => look.dark,
    };
    if dark { Mode::Dark } else { Mode::Light }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_follows_the_system_and_a_fixed_mode_does_not() {
        let dark = SysLook {
            dark: true,
            ..SysLook::default()
        };
        assert_eq!(mode_for(ModeChoice::Auto, &dark), Mode::Dark);
        assert_eq!(mode_for(ModeChoice::Auto, &SysLook::default()), Mode::Light);
        assert_eq!(mode_for(ModeChoice::Light, &dark), Mode::Light);
    }
}
