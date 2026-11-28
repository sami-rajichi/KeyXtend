//! Sensitive powers a module declares and the kernel grants by policy.

/// Declares `Capability` with `ALL` and `name`, so each variant and its name appear once.
macro_rules! capabilities {
    ($($(#[$doc:meta])* $variant:ident => $name:literal,)+) => {
        /// A sensitive power a module must declare and the kernel must grant.
        /// It is policy inside one process, not a sandbox (ADR-0005).
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Capability {
            $($(#[$doc])* $variant,)+
        }

        impl Capability {
            /// Every capability, in declaration order.
            pub const ALL: [Self; [$($name),+].len()] = [$(Self::$variant),+];

            /// The stable kebab-case name used by policy and logs.
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }
        }
    };
}

capabilities! {
    /// Send keystrokes and clicks to other apps.
    InjectInput => "inject-input",
    /// Watch the pointer through a system-wide hook.
    PointerHook => "pointer-hook",
    /// Read the clipboard.
    ReadClipboard => "read-clipboard",
    /// Set the clipboard and ask the keyboard for a paste.
    WriteClipboard => "write-clipboard",
    /// Read what is typed into the focused box, the words before its cursor, and its selection.
    ReadFocusedText => "read-focused-text",
    /// Hand the app's microphone pass to a recorder; the snip module alone holds it (ADR-0056, ADR-0058).
    Microphone => "microphone",
    /// Reach the network.
    Network => "network",
    /// Capture the screen.
    ScreenCapture => "screen-capture",
    /// Read and write the secret store.
    SecretStore => "secret-store",
    /// Scroll other apps' windows.
    Scroll => "scroll",
    /// Open the system's power dialog.
    Power => "power",
    /// Reach the keyboard's face and its pointer notes: the keyboard offers the face, the ui draws it, the mouse module
    /// sends the notes.
    KeyboardUi => "keyboard-ui",
    /// Carry Quick-fill's events that hold values; the keyboard, prediction and the ui hold it as an event gate.
    Vault => "vault",
    /// Reach Quick-fill's notebook store, its backup boxes and places; the vault module alone holds it (ADR-0039).
    VaultData => "vault-data",
    /// Carry the selection chips' events; the selection module, the keyboard and the ui hold it as an event gate
    /// (ADR-0041).
    Selection => "selection",
    /// Hand a link to the user's browser through the desktop shell; the selection and voice modules hold it (ADR-0041,
    /// ADR-0047).
    OpenLink => "open-link",
    /// Carry voice typing's events: the key, its light, the bar's view, its clicks and the text to type; the voice
    /// module, the keyboard and the ui hold it as an event gate (ADR-0042).
    Voice => "voice",
    /// Show a folder in the system's file manager: the voice module for its models folder (ADR-0046), Snip for its
    /// folder and to play a video (ADR-0056).
    OpenFolder => "open-folder",
    /// Ask where to save through the system's Save and folder dialogs; the snip module alone holds it (ADR-0056).
    SaveFile => "save-file",
    /// Carry Snip's events: the key, its light, the views, their clicks and the editor's line; the snip module, the
    /// keyboard and the ui hold it as an event gate, as `Voice` (ADR-0056).
    Snip => "snip",
    /// Hear the microphone and turn speech into text in the worker's speech jobs; the voice module alone holds it
    /// (ADR-0058).
    Speech => "speech",
}

/// True for a kebab-case name, as capability and event names must be.
#[cfg(test)]
pub(crate) fn is_kebab(name: &str) -> bool {
    let edges_ok = !name.starts_with('-') && !name.ends_with('-');
    let chars_ok = name.chars().all(|c| c.is_ascii_lowercase() || c == '-');
    !name.is_empty() && edges_ok && chars_ok && !name.contains("--")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn names_are_kebab_case() {
        for cap in Capability::ALL {
            assert!(is_kebab(cap.name()), "{cap:?} has name {}", cap.name());
        }
    }

    #[test]
    fn names_are_unique() {
        let names: HashSet<_> = Capability::ALL.iter().map(|c| c.name()).collect();
        assert_eq!(names.len(), Capability::ALL.len());
    }

    #[test]
    fn all_lists_each_variant_once() {
        let caps: HashSet<_> = Capability::ALL.iter().collect();
        assert_eq!(caps.len(), Capability::ALL.len());
        assert!(caps.contains(&Capability::SecretStore));
    }

    #[test]
    fn scroll_is_its_own_capability() {
        assert!(Capability::ALL.contains(&Capability::Scroll));
        assert_eq!(Capability::Scroll.name(), "scroll");
    }

    #[test]
    fn power_is_its_own_capability() {
        assert!(Capability::ALL.contains(&Capability::Power));
        assert_eq!(Capability::Power.name(), "power");
    }

    #[test]
    fn write_clipboard_follows_read_clipboard() {
        let write = Capability::WriteClipboard;
        assert_eq!(write.name(), "write-clipboard");
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        assert_eq!(at(write), at(Capability::ReadClipboard).map(|i| i + 1));
    }

    #[test]
    fn keyboard_ui_is_its_own_capability() {
        let ui = Capability::KeyboardUi;
        assert_eq!(ui.name(), "keyboard-ui");
        let others: Vec<_> = Capability::ALL.into_iter().filter(|&c| c != ui).collect();
        assert_eq!(others.len(), Capability::ALL.len() - 1, "listed once");
        assert!(others.iter().all(|c| c.name() != ui.name()), "{others:?}");
    }

    #[test]
    fn vault_then_vault_data_follow_the_older_ones() {
        let (vault, data) = (Capability::Vault, Capability::VaultData);
        assert_eq!((vault.name(), data.name()), ("vault", "vault-data"));
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        let ui = at(Capability::KeyboardUi);
        assert_eq!(
            (at(vault), at(data)),
            (ui.map(|i| i + 1), ui.map(|i| i + 2))
        );
        for cap in [vault, data] {
            let others = Capability::ALL.iter().filter(|&&c| c != cap);
            assert_eq!(others.count(), Capability::ALL.len() - 1, "listed once");
        }
    }

    #[test]
    fn selection_then_open_link_follow_vault_data() {
        let (sel, link) = (Capability::Selection, Capability::OpenLink);
        assert_eq!((sel.name(), link.name()), ("selection", "open-link"));
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        let data = at(Capability::VaultData);
        assert_eq!(
            (at(sel), at(link)),
            (data.map(|i| i + 1), data.map(|i| i + 2)),
            "older ones keep their places"
        );
        for cap in [sel, link] {
            let others = Capability::ALL.iter().filter(|&&c| c != cap);
            assert_eq!(others.count(), Capability::ALL.len() - 1, "listed once");
        }
    }

    #[test]
    fn open_folder_follows_voice() {
        let folder = Capability::OpenFolder;
        assert_eq!(folder.name(), "open-folder");
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        let voice = at(Capability::Voice);
        assert_eq!(at(folder), voice.map(|i| i + 1), "ADR-0046");
        let others = Capability::ALL.iter().filter(|&&c| c != folder);
        assert_eq!(others.count(), Capability::ALL.len() - 1, "listed once");
    }

    #[test]
    fn save_file_follows_open_folder_and_is_listed_once() {
        let save = Capability::SaveFile;
        assert_eq!(save.name(), "save-file");
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        let folder = at(Capability::OpenFolder);
        assert_eq!(at(save), folder.map(|i| i + 1), "ADR-0056 decision 10");
        let others = Capability::ALL.iter().filter(|&&c| c != save);
        assert_eq!(others.count(), Capability::ALL.len() - 1, "listed once");
    }

    #[test]
    fn snip_follows_save_file_and_is_listed_once() {
        let snip = Capability::Snip;
        assert_eq!(snip.name(), "snip");
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        let save = at(Capability::SaveFile);
        assert_eq!(at(snip), save.map(|i| i + 1), "an event gate, as Voice");
        let others = Capability::ALL.iter().filter(|&&c| c != snip);
        assert_eq!(others.count(), Capability::ALL.len() - 1, "listed once");
    }

    #[test]
    fn speech_follows_snip_and_is_listed_once() {
        let speech = Capability::Speech;
        assert_eq!(speech.name(), "speech");
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        let snip = at(Capability::Snip);
        assert_eq!(at(speech), snip.map(|i| i + 1), "ADR-0058");
        let others = Capability::ALL.iter().filter(|&&c| c != speech);
        assert_eq!(others.count(), Capability::ALL.len() - 1, "listed once");
    }

    #[test]
    fn voice_follows_open_link() {
        let voice = Capability::Voice;
        assert_eq!(voice.name(), "voice");
        let at = |cap| Capability::ALL.iter().position(|&c| c == cap);
        let link = at(Capability::OpenLink);
        assert_eq!(at(voice), link.map(|i| i + 1), "ADR-0042 decision 7");
        let others = Capability::ALL.iter().filter(|&&c| c != voice);
        assert_eq!(others.count(), Capability::ALL.len() - 1, "listed once");
    }
}
