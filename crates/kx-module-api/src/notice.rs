//! Kernel notices for the user, the answers to those that ask, and lifecycle events for the UI.

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
    /// The question's two answers when it asks; a plain notice has none. A reference keeps notices small.
    pub ask: Option<&'static NoticeAsk>,
}

impl Notice {
    /// Notice `key` about no module, with no arguments and no question.
    #[must_use]
    pub const fn plain(key: &'static str) -> Self {
        Self {
            key,
            module: None,
            args: Vec::new(),
            ask: None,
        }
    }

    /// Notice `key` about `module`, which its text may name as `$module`; it asks nothing.
    #[must_use]
    pub fn about(key: &'static str, module: ModuleId) -> Self {
        Self {
            module: Some(module),
            args: vec![(args::MODULE, NoticeArg::Text(module.to_string()))],
            ..Self::plain(key)
        }
    }
}

/// The translation keys of an asking notice's two buttons.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NoticeAsk {
    /// The main button's text, which answers Yes.
    pub yes: &'static str,
    /// The other button's text, which answers No, as closing the notice does.
    pub no: &'static str,
}

impl NoticeAsk {
    /// The plain Yes and No buttons.
    pub const YES_NO: Self = Self {
        yes: "button.yes",
        no: "button.no",
    };

    /// A memory check's Check and Later.
    pub const CHECK_LATER: Self = Self {
        yes: "button.check",
        no: "button.later",
    };

    /// An offer's Yes and Later.
    pub const YES_LATER: Self = Self {
        yes: Self::YES_NO.yes,
        no: Self::CHECK_LATER.no,
    };
}

/// The user answered an asking notice; its raiser knows it by the whole notice, args included.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoticeAnswered {
    /// The notice answered.
    pub notice: Notice,
    /// The user chose Yes.
    pub yes: bool,
}

impl Event for NoticeAnswered {
    const NAME: &'static str = "notice-answered";
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
    /// A language, by the name the layouts give it.
    pub const LANG: &str = "lang";
    /// Another app's short name, such as the one a hidden run started from.
    pub const APP: &str = "app";
}

impl Event for Notice {
    const NAME: &'static str = "notice";
}

/// Translation keys of the notices the kernel, the modules and the app show.
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
    /// The OS refused the mouse module's hook, so its modes stay off; the next toggle tries again.
    pub const MOUSE_HOOK_FAILED: &str = "notice.mouse-hook-failed";
    /// An error turned the mouse modes off for this session; a tap on a mode key starts them again.
    pub const MOUSE_RESET: &str = "notice.mouse-reset";
    /// The mouse module stopped after an error; it comes back when the app restarts.
    pub const MOUSE_STOPPED: &str = "notice.mouse-stopped";
    /// A pad tap came before the pointer left the keyboard, so there is nothing to scroll yet.
    pub const SCROLL_POINT: &str = "notice.scroll-point";
    /// Nothing at the pointed place can scroll, or its window closed.
    pub const SCROLL_NONE: &str = "notice.scroll-none";
    /// The pointed target is already at its end that way.
    pub const SCROLL_END: &str = "notice.scroll-end";
    /// The platform's scroll target did not open at a pad tap; the next tap tries again.
    pub const SCROLL_FAILED: &str = "notice.scroll-failed";
    /// The scroll module stopped after an error; it comes back when the app restarts.
    pub const SCROLL_STOPPED: &str = "notice.scroll-stopped";
    /// The system's power dialog did not open.
    pub const POWER_FAILED: &str = "notice.power-failed";
    /// The power module stopped after an error; it comes back when the app restarts.
    pub const POWER_STOPPED: &str = "notice.power-stopped";
    /// The learned words or emails were damaged, so learning starts again from empty.
    pub const WORDS_DAMAGED: &str = "notice.words-damaged";
    /// A full language's word pack is missing, so its suggestions come only from the words typed.
    pub const WORDS_MISSING: &str = "notice.words-missing";
    /// The learned words could not be unlocked, so nothing learned is saved this run.
    pub const WORDS_LOCKED: &str = "notice.words-locked";
    /// Part of the clipboard history was damaged, so those entries were left out.
    pub const CLIP_DAMAGED: &str = "notice.clip-damaged";
    /// The clipboard history could not be unlocked, so copies are not kept this run.
    pub const CLIP_LOCKED: &str = "notice.clip-locked";
    /// The run was started from inside another app, whose copy would hide its saves, so it saves nothing.
    pub const HIDDEN_RUN: &str = "notice.hidden-run";
    /// Asks whether to delete a hidden copy of our data in another app's folder.
    pub const HIDDEN_COPY: &str = "notice.hidden-copy";
    /// A hidden copy could not be deleted; it is asked about again next run.
    pub const HIDDEN_COPY_FAILED: &str = "notice.hidden-copy-failed";
    /// Quick-fill's saved entries were damaged and set aside; a backup can bring them back.
    pub const VAULT_DAMAGED: &str = "notice.vault-damaged";
    /// Asks whether the user still remembers the backup password, with Check and Later.
    pub const VAULT_CHECK: &str = "notice.vault-check";
    /// Offers a backup after the first entry is saved, with Yes and Later.
    pub const VAULT_OFFER: &str = "notice.vault-offer";
    /// Quick-fill's backup places and padlock were damaged and set aside, so the backup must be set up again.
    pub const VAULT_BACKUP_DAMAGED: &str = "notice.vault-backup-damaged";

    /// Every notice key, so the UI can check that each has its text.
    pub const ALL_KEYS: [&str; 31] = [
        SETTINGS_RESET,
        SETTINGS_RESTORED,
        SETTINGS_DEFAULTS,
        SETTINGS_NEWER,
        SETTINGS_UNSAVED,
        SETTINGS_LOCKED,
        MODULE_FAILED,
        KEYS_NOT_SENT,
        LANG_NOT_SWITCHED,
        MOUSE_HOOK_FAILED,
        MOUSE_RESET,
        MOUSE_STOPPED,
        SCROLL_POINT,
        SCROLL_NONE,
        SCROLL_END,
        SCROLL_FAILED,
        SCROLL_STOPPED,
        POWER_FAILED,
        POWER_STOPPED,
        WORDS_DAMAGED,
        WORDS_MISSING,
        WORDS_LOCKED,
        CLIP_DAMAGED,
        CLIP_LOCKED,
        HIDDEN_RUN,
        HIDDEN_COPY,
        HIDDEN_COPY_FAILED,
        VAULT_DAMAGED,
        VAULT_CHECK,
        VAULT_OFFER,
        VAULT_BACKUP_DAMAGED,
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
mod tests;
