//! Names of modules and services: static strings compared by value.

use std::fmt;

/// Declares a name type over `&'static str` with `new`, `as_str` and `Display`.
macro_rules! static_name {
    ($(#[$doc:meta])* $name:ident, $example:literal) => {
        $(#[$doc])*
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(&'static str);

        impl $name {
            #[doc = concat!("Wraps `name`, such as `\"", $example, "\"`.")]
            #[must_use]
            pub const fn new(name: &'static str) -> Self {
                Self(name)
            }

            /// The name as text.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.pad(self.0)
            }
        }
    };
}

static_name!(
    /// A module's unique name.
    ModuleId,
    "keyboard"
);

static_name!(
    /// A service's unique name.
    ServiceId,
    "input-injector"
);

#[cfg(test)]
mod tests {
    use super::*;

    const KEYBOARD: ModuleId = ModuleId::new("keyboard");
    const INJECTOR: ServiceId = ServiceId::new("input-injector");

    #[test]
    fn display_prints_the_name() {
        assert_eq!(KEYBOARD.to_string(), "keyboard");
        assert_eq!(INJECTOR.to_string(), "input-injector");
    }

    #[test]
    fn as_str_returns_the_name() {
        assert_eq!(KEYBOARD.as_str(), "keyboard");
        assert_eq!(INJECTOR.as_str(), "input-injector");
    }

    #[test]
    fn ids_compare_by_name() {
        assert_eq!(ModuleId::new("a"), ModuleId::new("a"));
        assert!(ServiceId::new("a") < ServiceId::new("b"));
    }
}
