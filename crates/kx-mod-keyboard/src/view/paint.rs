//! Which palette tokens paint a key: its fill, legend ink and Dolch skirt (mock-up lines 168-176 and 203-208).
//! They are `kx-theme` palette names as text, so the keyboard needs no theme crate.

use crate::spec::KeyKind;
use serde::Serialize;

/// Token names that paint one key.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Paint {
    /// Cap colour, a palette token.
    pub fill: &'static str,
    /// Legend colour, a palette token.
    pub ink: &'static str,
    /// Skirt colour, a `skirt` token of sculpted themes.
    pub skirt: &'static str,
}

/// The paint of a key that comes in a later version (`shape.toml [dim]`).
pub const DIM: Paint = Paint {
    fill: "dim",
    ink: "dim_ink",
    skirt: "dim",
};

/// The paint of a key of `kind`: side keys turn `on` when toggled, other keys `lock` when latched.
#[must_use]
pub fn paint(kind: KeyKind, on: bool) -> Paint {
    let (fill, ink, skirt) = match (on, kind) {
        (true, KeyKind::Act) => ("on", "on_ink", "on"),
        (true, _) => ("lock", "lock_ink", "on"),
        (_, KeyKind::Char | KeyKind::Space) => ("key", "legend", "key"),
        (_, KeyKind::Mod | KeyKind::Lang) => ("key_mod", "legend_mod", "key_mod"),
        (_, KeyKind::Act) => ("key_act", "legend_act", "key_mod"),
        (_, KeyKind::Enter) => ("enter", "enter_ink", "enter"),
        (_, KeyKind::Danger) => ("danger", "danger_ink", "danger"),
    };
    Paint { fill, ink, skirt }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latched_keys_lock_and_toggled_side_keys_turn_on() {
        assert_eq!(paint(KeyKind::Mod, true).fill, "lock");
        assert_eq!(paint(KeyKind::Act, true).fill, "on");
        assert_eq!(paint(KeyKind::Act, false).skirt, "key_mod");
        assert_eq!(paint(KeyKind::Enter, false).ink, "enter_ink");
    }

    #[test]
    fn a_dimmed_key_takes_the_dim_tokens() {
        assert_eq!((DIM.fill, DIM.ink, DIM.skirt), ("dim", "dim_ink", "dim"));
    }
}
