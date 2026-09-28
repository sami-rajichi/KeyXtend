//! Which palette tokens paint a key: its fill, legend ink and Dolch skirt (mock-up lines 168-176 and 203-208).

use serde::Serialize;

use crate::kbgeom::KeyKind;

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

/// The paint of a recording key (mock-up `.key.rec`).
pub const REC: Paint = Paint {
    fill: "rec",
    ink: "rec_ink",
    skirt: "rec",
};

/// The paint of a key of `kind`: side keys turn `on` when toggled, other keys `lock` when latched.
pub fn paint(kind: KeyKind, side: bool, on: bool) -> Paint {
    let (fill, ink, skirt) = match (on, side, kind) {
        (true, true, _) => ("on", "on_ink", "on"),
        (true, false, _) => ("lock", "lock_ink", "on"),
        (_, _, KeyKind::Char | KeyKind::Space) => ("key", "legend", "key"),
        (_, _, KeyKind::Mod | KeyKind::Lang) => ("key_mod", "legend_mod", "key_mod"),
        (_, _, KeyKind::Act) => ("key_act", "legend_act", "key_mod"),
        (_, _, KeyKind::Enter) => ("enter", "enter_ink", "enter"),
        (_, _, KeyKind::Danger) => ("danger", "danger_ink", "danger"),
    };
    Paint { fill, ink, skirt }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [KeyKind; 7] = [
        KeyKind::Char,
        KeyKind::Mod,
        KeyKind::Danger,
        KeyKind::Enter,
        KeyKind::Lang,
        KeyKind::Space,
        KeyKind::Act,
    ];

    #[test]
    fn every_paint_names_tokens_each_theme_has() {
        for t in &crate::theme::load().expect("themes load").list {
            let p = &t.light;
            let skirt = serde_json::to_value(p.skirt.as_ref()).expect("skirt");
            assert!(p.get(REC.fill).is_some() && p.get(REC.ink).is_some());
            assert!(skirt.is_null() || skirt[REC.skirt].is_string(), "rec");
            for (kind, side, on) in KINDS.iter().flat_map(|&k| {
                [
                    (k, false, false),
                    (k, true, false),
                    (k, false, true),
                    (k, true, true),
                ]
            }) {
                let c = paint(kind, side, on);
                assert!(p.get(c.fill).is_some() && p.get(c.ink).is_some(), "{c:?}");
                assert!(skirt.is_null() || skirt[c.skirt].is_string(), "{c:?}");
            }
        }
    }

    #[test]
    fn latched_keys_lock_and_toggled_side_keys_turn_on() {
        assert_eq!(paint(KeyKind::Mod, false, true).fill, "lock");
        assert_eq!(paint(KeyKind::Act, true, true).fill, "on");
        assert_eq!(paint(KeyKind::Act, true, false).skirt, "key_mod");
        assert_eq!(paint(KeyKind::Enter, false, false).ink, "enter_ink");
    }
}
