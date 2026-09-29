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
    /// Read the text of the focused field in another app.
    ReadFocusedText => "read-focused-text",
    /// Record from the microphone.
    Microphone => "microphone",
    /// Reach the network.
    Network => "network",
    /// Capture the screen.
    ScreenCapture => "screen-capture",
    /// Read and write the secret store.
    SecretStore => "secret-store",
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn is_kebab(name: &str) -> bool {
        let edges_ok = !name.starts_with('-') && !name.ends_with('-');
        let chars_ok = name.chars().all(|c| c.is_ascii_lowercase() || c == '-');
        !name.is_empty() && edges_ok && chars_ok && !name.contains("--")
    }

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
}
