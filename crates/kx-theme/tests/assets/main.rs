//! The fonts and icons the tokens name exist in the repo's `assets/` folder. The app checks that named-key icons name
//! real board keys, as only it sees the board.

#![allow(
    clippy::expect_used,
    reason = "these helpers run only in tests, where a failed setup should fail the test"
)]

mod cmap;
mod fontname;

use std::path::{Path, PathBuf};

use kx_theme::Themes;
use kx_theme::assets::{
    FONT_DIR, ICON_EXT, ICONS_AT, SYSTEM_FONTS, bundle, font_files, icon_file, is_font,
};
use std::collections::BTreeSet;

/// The repo's `assets/` folder.
fn assets() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

/// The circle marks with no letter before them sit on (U+25CC).
const CIRCLE: char = '\u{25CC}';
/// Mark fonts that ship with Windows, so no file here can be read, and why each draws the circle.
const SYSTEM_MARK_FONTS: [(&str, &str); 1] = [("Segoe UI", "Windows 11's Segoe UI holds U+25CC")];

/// The bytes of every bundled font file that names itself `family`.
fn files_of(family: &str) -> Vec<Vec<u8>> {
    let files = font_files(&assets()).expect("assets/fonts reads");
    let read = files
        .iter()
        .map(|p| std::fs::read(p).expect("a font reads"));
    read.filter(|b| fontname::families(b).iter().any(|f| f == family))
        .collect()
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
        let fonts = [
            &th.look.latin_fonts,
            &th.look.arabic_fonts,
            &th.look.mark_fonts,
        ];
        for family in fonts.into_iter().flatten() {
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
fn every_mark_font_a_theme_names_draws_the_dotted_circle() {
    let t = Themes::shipped().expect("tokens load");
    let mut checked = 0;
    for th in &t.list {
        let id = &th.look.id;
        for family in &th.look.mark_fonts {
            if SYSTEM_MARK_FONTS.iter().any(|(f, _)| f == family) {
                continue;
            }
            let files = files_of(family);
            assert!(!files.is_empty(), "{id}: {family} is not bundled or listed");
            for f in &files {
                assert!(cmap::maps(f, CIRCLE), "{id}: {family} has no U+25CC");
            }
            checked += 1;
        }
    }
    assert!(checked > 0, "the check reached the bundled fonts");
}

#[test]
fn the_circle_check_reads_real_fonts_rubik_lacking_it_and_noto_holding_it() {
    let rubik = files_of("Rubik");
    assert!(!rubik.is_empty(), "Rubik is bundled");
    for f in &rubik {
        assert!(
            !cmap::maps(f, CIRCLE),
            "why Dolch's marks need their own font"
        );
        assert!(
            cmap::maps(f, 'A') && cmap::maps(f, '\u{0636}'),
            "its letters read"
        );
    }
    let noto = files_of("Noto Sans Arabic");
    assert!(!noto.is_empty() && noto.iter().all(|f| cmap::maps(f, CIRCLE)));
}

#[test]
fn the_bundle_holds_each_named_icon_once_and_every_font_under_url_safe_names() {
    let b = bundle(&assets()).expect("the bundle is found");
    let t = Themes::shipped().expect("tokens load");
    let names: BTreeSet<&str> = t.icons.names().into_iter().collect();
    let icons: Vec<String> = names
        .iter()
        .map(|n| format!("{ICONS_AT}/{n}.{ICON_EXT}"))
        .collect();
    let bundled: Vec<&str> = b.icons.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(bundled, icons, "each named icon once, flattened");
    assert!(b.icons.iter().all(|f| f.file.is_file()));
    let fonts = font_files(&assets()).expect("assets/fonts reads");
    let files: Vec<&PathBuf> = b.fonts.iter().map(|f| &f.file).collect();
    assert_eq!(files, fonts.iter().collect::<Vec<_>>(), "every font");
    let safe = |c: char| c.is_ascii_alphanumeric() || "./-_".contains(c);
    for f in &b.fonts {
        assert!(
            f.name.starts_with(FONT_DIR) && f.name.chars().all(safe),
            "{}",
            f.name
        );
    }
    let unique: BTreeSet<&str> = b.fonts.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(unique.len(), b.fonts.len(), "no two fonts share a name");
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
