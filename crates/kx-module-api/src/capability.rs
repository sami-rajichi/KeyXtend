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
    /// Reads what is typed into the focused box, and the words before its cursor.
    ReadFocusedText => "read-focused-text",
    /// Record from the microphone.
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
    /// Draws the keyboard and hears its pointer notes.
    KeyboardUi => "keyboard-ui",
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
    fn keyboard_ui_is_its_own_capability() {
        let ui = Capability::KeyboardUi;
        assert_eq!(ui.name(), "keyboard-ui");
        let others: Vec<_> = Capability::ALL.into_iter().filter(|&c| c != ui).collect();
        assert_eq!(others.len(), Capability::ALL.len() - 1, "listed once");
        assert!(others.iter().all(|c| c.name() != ui.name()), "{others:?}");
    }
}
