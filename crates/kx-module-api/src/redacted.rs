//! A wrapper that keeps sensitive values out of logs and debug output, and the secrets that are also wiped.

use std::fmt;
use zeroize::Zeroizing;

/// What a `Redacted` value prints in place of its content.
pub const REDACTED_MARKER: &str = "‹redacted›";

/// A secret text, such as a saved value or a password: it never prints, and it is wiped when dropped.
pub type Secret = Redacted<Zeroizing<String>>;
/// Secret bytes, such as a box's opened part: they never print, and they are wiped when dropped.
pub type SecretBytes = Redacted<Zeroizing<Vec<u8>>>;

/// A sensitive value (typed text, clipboard, secret, transcript) that never prints itself.
///
/// It has no serde impls, so it cannot be serialized by accident.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
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
    fn format_flags_print_exactly_the_marker() {
        assert_eq!(format!("{:>20}", secret()), REDACTED_MARKER);
        assert_eq!(format!("{:.2}", secret()), REDACTED_MARKER);
        assert_eq!(format!("{:>20?}", secret()), REDACTED_MARKER);
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
    fn a_secret_prints_only_the_marker_and_is_held_to_be_wiped() {
        let secret: Secret = Redacted::new(Zeroizing::new(SECRET.to_owned()));
        let held: &Zeroizing<String> = secret.expose();
        assert_eq!(held.as_str(), SECRET);
        assert_eq!(format!("{secret:?}{secret}"), REDACTED_MARKER.repeat(2));
        let bytes: SecretBytes = Redacted::new(Zeroizing::new(SECRET.as_bytes().to_vec()));
        assert_eq!(format!("{bytes:?}"), REDACTED_MARKER);
    }

    #[test]
    fn a_plain_number_stays_copy_and_still_prints_only_the_marker() {
        let caret = Redacted::new(7_u32);
        let (first, second) = (caret, caret);
        assert_eq!(first, second);
        assert_eq!(format!("{first:?}"), REDACTED_MARKER);
    }

    #[test]
    fn clone_eq_and_default_follow_the_value() {
        assert_eq!(secret().clone(), secret());
        assert_ne!(secret(), Redacted::new(String::new()));
        assert_eq!(Redacted::<String>::default().expose(), "");
    }
}
