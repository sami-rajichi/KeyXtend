//! Kernel notices for the user, and lifecycle events for the UI.

use crate::{Event, ModuleId};

/// A message for the user, named by a translation key; `kx-ui` holds the texts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// The translation key, one of `keys`.
    pub key: &'static str,
    /// The module the notice is about, if any.
    pub module: Option<ModuleId>,
    /// Named values for the text, by the names in `args`.
    pub args: Vec<(&'static str, NoticeArg)>,
}

/// One value for a notice's text; never typed text, clipboard data or secrets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NoticeArg {
    /// A word shown as it is, or a module id the UI shows by its name.
    Text(String),
    /// A number, so the text can take the right plural form.
    Count(usize),
    /// Items the UI joins in its language, such as setting keys.
    List(Vec<String>),
}

/// The names of notice arguments, which the translations use.
pub mod args {
    /// The module the notice is about, as a module id.
    pub const MODULE: &str = "module";
    /// The setting keys that went back to their defaults, as a list.
    pub const KEYS: &str = "keys";
    /// How many items the notice's list leaves out; absent when it names them all.
    pub const MORE: &str = "more";
}

impl Event for Notice {
    const NAME: &'static str = "notice";
}

/// Translation keys of the notices the kernel and the keyboard face show.
pub mod keys {
    /// Some settings values were bad and went back to their defaults.
    pub const SETTINGS_RESET: &str = "notice.settings-reset";
    /// The settings file was damaged, so the previous copy was restored.
    pub const SETTINGS_RESTORED: &str = "notice.settings-restored";
    /// No settings file was usable, so the defaults are in use.
    pub const SETTINGS_DEFAULTS: &str = "notice.settings-defaults";
    /// A settings section comes from a newer version; it is kept and defaults are used.
    pub const SETTINGS_NEWER: &str = "notice.settings-newer";
    /// Settings could not be saved and live in memory only.
    pub const SETTINGS_UNSAVED: &str = "notice.settings-unsaved";
    /// Another program holds the settings file, so defaults run in memory and nothing is saved.
    pub const SETTINGS_LOCKED: &str = "notice.settings-locked";
    /// A module failed while the rest keep running; it can be tried again.
    pub const MODULE_FAILED: &str = "notice.module-failed";
    /// A click's keys or text did not all reach the app in front.
    pub const KEYS_NOT_SENT: &str = "notice.keys-not-sent";
    /// The app in front could not be asked to switch language.
    pub const LANG_NOT_SWITCHED: &str = "notice.lang-not-switched";

    /// Every notice key, so the UI can check that each has its text.
    pub const ALL_KEYS: [&str; 9] = [
        SETTINGS_RESET,
        SETTINGS_RESTORED,
        SETTINGS_DEFAULTS,
        SETTINGS_NEWER,
        SETTINGS_UNSAVED,
        SETTINGS_LOCKED,
        MODULE_FAILED,
        KEYS_NOT_SENT,
        LANG_NOT_SWITCHED,
    ];
}

/// Where a module is in its lifecycle; the kernel drives it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleState {
    /// Waiting for its turn to start.
    Pending,
    /// Running its `start`.
    Starting,
    /// Started and running.
    Active,
    /// Its start failed or it panicked; the rest keep running.
    Failed,
    /// Running its `stop`.
    Stopping,
    /// Stopped.
    Stopped,
}

/// A module moved to a new lifecycle state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModuleStateChanged {
    /// The module that moved.
    pub module: ModuleId,
    /// Its new state.
    pub state: ModuleState,
}

impl Event for ModuleStateChanged {
    const NAME: &'static str = "module-state-changed";
}

/// A module's settings changed; the kernel restarts the module if it runs, or tries it again if it
/// failed, so it reads them again.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SettingsChanged {
    /// The module whose settings changed.
    pub module: ModuleId,
}

impl Event for SettingsChanged {
    const NAME: &'static str = "settings-changed";
}

#[cfg(test)]
mod tests {
    use super::*;
    use keys::ALL_KEYS;
    use std::collections::HashSet;

    #[test]
    fn keys_are_unique_notice_keys() {
        let unique: HashSet<_> = ALL_KEYS.iter().collect();
        assert_eq!(unique.len(), ALL_KEYS.len());
        assert!(ALL_KEYS.iter().all(|k| k.starts_with("notice.")));
    }

    #[test]
    fn arguments_keep_the_names_translations_use() {
        assert_eq!(
            (args::MODULE, args::KEYS, args::MORE),
            ("module", "keys", "more")
        );
    }

    #[test]
    fn event_names_are_distinct() {
        let names = [
            Notice::NAME,
            ModuleStateChanged::NAME,
            SettingsChanged::NAME,
        ];
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len());
    }

    #[test]
    fn a_state_change_carries_module_and_state() {
        let module = ModuleId::new("keyboard");
        let change = ModuleStateChanged {
            module,
            state: ModuleState::Failed,
        };
        assert_eq!((change.module, change.state), (module, ModuleState::Failed));
    }
}
