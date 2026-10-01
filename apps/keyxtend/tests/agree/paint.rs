//! Every paint the keyboard gives a key names palette tokens each theme has, light and dark (moved from the spike's
//! `kbctl` tests, since only the app sees both the keyboard and the themes).

use kx_mod_keyboard::KeyKind;
use kx_mod_keyboard::view::{DIM, Paint, paint};
use kx_theme::Themes;

/// Every key kind.
fn kinds() -> [KeyKind; 7] {
    // Exhaustive on purpose: a new kind stops this building until it is listed here.
    let listed = |k: KeyKind| match k {
        KeyKind::Char
        | KeyKind::Mod
        | KeyKind::Danger
        | KeyKind::Enter
        | KeyKind::Lang
        | KeyKind::Space
        | KeyKind::Act => k,
    };
    [
        KeyKind::Char,
        KeyKind::Mod,
        KeyKind::Danger,
        KeyKind::Enter,
        KeyKind::Lang,
        KeyKind::Space,
        KeyKind::Act,
    ]
    .map(listed)
}

#[test]
fn every_paint_names_tokens_each_theme_has() {
    let lit = kinds()
        .into_iter()
        .flat_map(|k| [paint(k, false), paint(k, true)]);
    let all: Vec<Paint> = lit.chain([DIM]).collect();
    for t in &Themes::shipped().unwrap().list {
        for p in [&t.light, &t.dark] {
            let skirt = serde_json::to_value(p.skirt.as_ref()).unwrap();
            for c in &all {
                let id = &t.look.id;
                assert!(
                    p.get(c.fill).is_some() && p.get(c.ink).is_some(),
                    "{id}: {c:?}"
                );
                assert!(skirt.is_null() || skirt[c.skirt].is_string(), "{id}: {c:?}");
            }
        }
    }
}
