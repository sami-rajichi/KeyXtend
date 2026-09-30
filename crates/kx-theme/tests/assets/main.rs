//! The fonts and icons the tokens name exist in the repo's `assets/` folder, and named-key icons name real keys.

#![allow(
    clippy::expect_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod fontname;

use std::path::{Path, PathBuf};

use kx_theme::Themes;
use kx_theme::assets::{font_files, icon_file, is_font};

/// Families Windows 11 ships, so the app does not bundle them.
const SYSTEM_FONTS: [&str; 2] = ["Segoe UI Variable Text", "Segoe UI"];
/// The keyboard's board, whose key ids `icons.toml [keys]` uses.
const BOARD: &str = "../kx-mod-keyboard/board.toml";

/// The repo's `assets/` folder.
fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

/// The family names inside every font file in `assets/fonts/`, one folder per family.
fn bundled_families() -> Vec<String> {
    let files = font_files(&assets()).expect("assets/fonts reads");
    files
        .iter()
        .flat_map(|p| fontname::families(&std::fs::read(p).expect("a font reads")))
        .collect()
}

#[test]
fn the_font_list_holds_only_font_files_from_every_family() {
    let files = font_files(&assets()).expect("assets/fonts reads");
    assert!(files.iter().all(|p| is_font(p)), "{files:?}");
    let families = std::fs::read_dir(assets().join("fonts")).expect("assets/fonts reads");
    for family in families.flatten().filter(|d| d.path().is_dir()) {
        let found = files.iter().any(|f| f.starts_with(family.path()));
        assert!(found, "no font file from {}", family.path().display());
    }
    assert!(!is_font(Path::new("OFL.txt")), "licence files stay out");
}

#[test]
fn every_font_a_theme_names_is_bundled_or_ships_with_windows() {
    let t = Themes::shipped().expect("tokens load");
    let bundled = bundled_families();
    let mut checked = 0;
    for th in &t.list {
        for family in th.look.latin_fonts.iter().chain(&th.look.arabic_fonts) {
            if SYSTEM_FONTS.contains(&family.as_str()) {
                continue;
            }
            let id = &th.look.id;
            assert!(
                bundled.contains(family),
                "{id}: no font file names itself {family}"
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "the check reached the bundled fonts");
}

#[test]
fn every_icon_name_has_a_file() {
    let t = Themes::shipped().expect("tokens load");
    for name in t.icons.names() {
        assert!(
            icon_file(&assets(), name).is_some(),
            "no icon file for {name}"
        );
    }
    assert!(icon_file(&assets(), "no-such-icon").is_none());
}

/// Every string at an `id` key in `v`, at any depth.
fn ids(v: &toml::Value, out: &mut Vec<String>) {
    match v {
        toml::Value::Table(t) => {
            if let Some(toml::Value::String(id)) = t.get("id") {
                out.push(id.clone());
            }
            t.values().for_each(|v| ids(v, out));
        }
        toml::Value::Array(a) => a.iter().for_each(|v| ids(v, out)),
        _ => {}
    }
}

#[test]
fn every_named_key_icon_names_a_board_key() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(BOARD);
    let text = std::fs::read_to_string(&path).expect("board.toml reads");
    let board: toml::Value = toml::from_str(&text).expect("board.toml parses");
    let mut known = Vec::new();
    ids(&board, &mut known);
    let t = Themes::shipped().expect("tokens load");
    for id in t.icons.keys.keys() {
        assert!(known.contains(id), "icons.toml [keys] {id} is no board key");
    }
}
