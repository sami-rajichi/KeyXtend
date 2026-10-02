//! The running copy's look settings from `[ui]`, read as a later copy of the app reads them, without kx-ui's Qt.
#![cfg(windows)]

use kx_module_api::{ModuleId, Settings, SettingsSpec, validate_as};
use kx_settings::peek_section;
use kx_theme::Themes;
use serde::{Deserialize, Serialize};

use crate::product;

/// kx-ui's module id, which names its settings section.
const UI: ModuleId = ModuleId::new("ui");
/// The settings key that makes a theme take the Windows accent.
pub const FOLLOW_KEY: &str = "follow_accent";
/// The settings key of the theme.
pub const THEME_KEY: &str = "theme";

/// kx-ui's section, with kx-ui's own defaults file; a test ties its version to kx-ui's.
static SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: include_str!("../../../crates/kx-ui/defaults.toml"),
    migrations: &[],
    validate: validate_as::<UiLook>,
};

/// The `[ui]` settings the gates depend on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiLook {
    /// The theme, by its `themes.toml` id.
    pub theme: String,
    /// True when a theme that can take the Windows accent takes it.
    pub follow_accent: bool,
    /// What sits behind the keyboard.
    pub backdrop: String,
}

impl Settings for UiLook {}

impl UiLook {
    /// True when the keyboard takes the Windows accent: `follow_accent` on a theme that can.
    pub fn takes_accent(&self, themes: &Themes) -> bool {
        self.follow_accent && themes.find(&self.theme).is_some_and(|t| t.look.accent)
    }
}

/// The look in settings file `file`'s `[ui]` section, over kx-ui's defaults.
fn look_in(file: Option<&toml::Table>) -> Result<UiLook, String> {
    peek_section(file, UI, &SPEC).map_err(|e| format!("the [ui] defaults: {e}"))
}

/// The running copy's look; kx-ui's defaults when no copy runs.
pub fn running() -> Result<UiLook, String> {
    look_in(product::running_file()?.as_ref())
}

#[cfg(test)]
mod tests {
    use kx_theme::Themes;

    use super::*;

    fn table(text: &str) -> toml::Table {
        text.parse().expect("TOML")
    }

    #[test]
    fn the_section_and_its_version_are_kx_uis() {
        let module = include_str!("../../../crates/kx-ui/src/ui_module.rs");
        assert!(module.contains(&format!("ModuleId::new(\"{}\")", UI.as_str())));
        let settings = include_str!("../../../crates/kx-ui/src/settings.rs");
        assert!(settings.contains(&format!("version: {},", SPEC.version)));
    }

    #[test]
    fn the_look_is_the_files_ui_section_over_kx_uis_defaults() {
        let plain = look_in(None).expect("the defaults read");
        assert_eq!(
            (plain.follow_accent, plain.backdrop.as_str()),
            (false, "none")
        );
        let file =
            table("[ui]\nversion = 1\nfollow_accent = true\ntheme = \"dolch\"\nmode = \"dark\"");
        let set = look_in(Some(&file)).expect("reads");
        assert_eq!((set.follow_accent, set.theme.as_str()), (true, "dolch"));
        assert_eq!(set.backdrop, plain.backdrop, "the rest keeps its default");
    }

    #[test]
    fn the_keyboard_takes_the_accent_only_with_follow_accent_on_a_theme_that_can() {
        let themes = Themes::shipped().expect("the themes read");
        let can = themes
            .list
            .iter()
            .find(|t| t.look.accent)
            .expect("one theme takes it");
        let cannot = themes
            .list
            .iter()
            .find(|t| !t.look.accent)
            .expect("one does not");
        let look = |theme: &str, follow_accent| UiLook {
            theme: theme.to_string(),
            follow_accent,
            backdrop: String::new(),
        };
        assert!(look(&can.look.id, true).takes_accent(&themes));
        assert!(!look(&can.look.id, false).takes_accent(&themes));
        assert!(!look(&cannot.look.id, true).takes_accent(&themes));
        assert!(!look("no-such-theme", true).takes_accent(&themes));
    }

    #[test]
    fn the_keys_named_to_the_owner_are_the_settings_keys() {
        let keys = toml::Table::try_from(look_in(None).expect("reads")).expect("a table");
        assert!(keys.contains_key(FOLLOW_KEY) && keys.contains_key(THEME_KEY));
    }
}
