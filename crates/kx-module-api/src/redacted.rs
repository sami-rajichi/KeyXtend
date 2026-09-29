//! A wrapper that keeps sensitive values out of logs and debug output.

use std::fmt;

/// What a `Redacted` value prints in place of its content.
pub const REDACTED_MARKER: &str = "‹redacted›";

/// A sensitive value (typed text, clipboard, secret, transcript) that never prints itself.
///
/// It has no serde impls, so it cannot be serialized by accident.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    /// Wraps `value`.
    #[must_use]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// The wrapped value: the only way to read it without consuming the wrapper.
    #[must_use]
    pub const fn expose(&self) -> &T {
        &self.0
    }

    /// Unwraps the value.
    #[must_use]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> From<T> for Redacted<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_MARKER)
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_MARKER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "hunter2-s3cret";

    fn secret() -> Redacted<String> {
        Redacted::new(SECRET.to_owned())
    }

    #[test]
    fn debug_prints_only_the_marker() {
        assert_eq!(format!("{:?}", secret()), REDACTED_MARKER);
    }

    #[test]
    fn pretty_debug_prints_only_the_marker() {
        assert_eq!(format!("{:#?}", secret()), REDACTED_MARKER);
    }

    #[test]
    fn display_prints_only_the_marker() {
        assert_eq!(format!("{}", secret()), REDACTED_MARKER);
    }

    #[test]
    fn no_format_leaks_the_value() {
        let all = format!("{:?}{:#?}{}", secret(), secret(), secret());
        assert!(!all.contains(SECRET));
    }

    #[test]
    fn display_needs_no_display_for_the_value() {
        struct Opaque;
        assert_eq!(format!("{}", Redacted::new(Opaque)), REDACTED_MARKER);
    }

    #[test]
    fn a_derived_debug_struct_shows_the_marker_in_place() {
        #[derive(Debug)]
        struct Line {
            #[allow(dead_code, reason = "read only through Debug")]
            text: Redacted<String>,
        }
        let shown = format!("{:?}", Line { text: secret() });
        assert_eq!(shown, format!("Line {{ text: {REDACTED_MARKER} }}"));
    }

    #[test]
    fn a_nested_pretty_debug_hides_the_value() {
        let shown = format!("{:#?}", Some(secret()));
        assert!(shown.contains(REDACTED_MARKER));
        assert!(!shown.contains(SECRET));
    }

    #[test]
    fn expose_and_into_inner_return_the_value() {
        assert_eq!(secret().expose(), SECRET);
        assert_eq!(secret().into_inner(), SECRET);
    }

    #[test]
    fn from_wraps_the_value() {
        let r: Redacted<u8> = 7.into();
        assert_eq!(*r.expose(), 7);
    }

    #[test]
    fn clone_eq_and_default_follow_the_value() {
        assert_eq!(secret().clone(), secret());
        assert_ne!(secret(), Redacted::new(String::new()));
        assert_eq!(Redacted::<String>::default().expose(), "");
    }
}
