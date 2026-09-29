//! Kernel notices for the user, and lifecycle events for the UI.

use crate::{Event, ModuleId};

/// A message for the user, named by a translation key; P3 adds the texts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// The translation key, one of `keys`.
    pub key: &'static str,
    /// The module the notice is about, if any.
    pub module: Option<ModuleId>,
    /// Named values for the text; never typed text, clipboard data or secrets.
    pub args: Vec<(&'static str, String)>,
}

impl Event for Notice {
    const NAME: &'static str = "notice";
}

/// Translation keys of the kernel's notices.
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
    /// A module failed while the rest keep running; it can be tried again.
    pub const MODULE_FAILED: &str = "notice.module-failed";
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

/// A module's settings changed; the kernel restarts the module if it runs, so it reads them again.
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
    use std::collections::HashSet;

    const ALL_KEYS: [&str; 6] = [
        keys::SETTINGS_RESET,
        keys::SETTINGS_RESTORED,
        keys::SETTINGS_DEFAULTS,
        keys::SETTINGS_NEWER,
        keys::SETTINGS_UNSAVED,
        keys::MODULE_FAILED,
    ];

    #[test]
    fn keys_are_unique_notice_keys() {
        let unique: HashSet<_> = ALL_KEYS.iter().collect();
        assert_eq!(unique.len(), ALL_KEYS.len());
        assert!(ALL_KEYS.iter().all(|k| k.starts_with("notice.")));
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
