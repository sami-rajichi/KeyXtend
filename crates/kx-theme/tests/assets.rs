//! The fonts and icons the tokens name exist in the repo's `assets/` folder, and named-key icons name real keys.

use std::path::{Path, PathBuf};

use kx_theme::Themes;

/// Families Windows 11 ships, so the app does not bundle them.
const SYSTEM_FONTS: [&str; 2] = ["Segoe UI Variable Text", "Segoe UI"];
/// Font file types the app bundles.
const FONT_EXTS: [&str; 2] = ["ttf", "otf"];
/// Folders under `assets/` that hold icons: Lucide's, then our own drawings.
const ICON_DIRS: [&str; 2] = ["icons/lucide", "icons"];
/// The icon file type.
const ICON_EXT: &str = "svg";
/// The keyboard's board, whose key ids `icons.toml [keys]` uses.
const BOARD: &str = "../kx-mod-keyboard/board.toml";

/// The repo's `assets/` folder.
fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

/// The folder a bundled family lives in: its name in lower case, without spaces.
fn font_dir(family: &str) -> PathBuf {
    let name: String = family.split_whitespace().collect();
    assets().join("fonts").join(name.to_lowercase())
}

/// True when `dir` holds a font file.
fn has_font(dir: &Path) -> bool {
    let font = |p: &Path| {
        p.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| FONT_EXTS.iter().any(|x| e.eq_ignore_ascii_case(x)))
    };
    std::fs::read_dir(dir).is_ok_and(|d| d.flatten().any(|e| font(&e.path())))
}

#[test]
fn every_font_a_theme_names_is_bundled_or_ships_with_windows() {
    let t = Themes::shipped().expect("tokens load");
    let mut bundled = 0;
    for th in &t.list {
        for family in th.look.latin_fonts.iter().chain(&th.look.arabic_fonts) {
            if SYSTEM_FONTS.contains(&family.as_str()) {
                continue;
            }
            let dir = font_dir(family);
            assert!(
                has_font(&dir),
                "{}: no {family} in {}",
                th.look.id,
                dir.display()
            );
            bundled += 1;
        }
    }
    assert!(bundled > 0, "the check reached the bundled fonts");
}

#[test]
fn every_icon_name_has_a_file() {
    let t = Themes::shipped().expect("tokens load");
    for name in t.icons.names() {
        let file = |d: &str| assets().join(d).join(format!("{name}.{ICON_EXT}"));
        assert!(
            ICON_DIRS.iter().any(|d| file(d).is_file()),
            "no icon file for {name}"
        );
    }
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
