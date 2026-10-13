//! The font table (`font_files.toml`): each family a theme bundles maps to files that exist and name themselves that
//! family, under the names the bundle gives them, and every bundled file belongs to a family some theme names.

use crate::{assets, fontname};
use kx_theme::Themes;
use kx_theme::assets::{FontFiles, SYSTEM_FONTS, bundle, font_files};
use std::collections::BTreeSet;

/// Every family the themes name, Latin, Arabic and marks, once.
fn named() -> BTreeSet<String> {
    let t = Themes::shipped().expect("tokens load");
    let lists = t.list.iter().flat_map(|th| {
        let look = &th.look;
        [&look.latin_fonts, &look.arabic_fonts, &look.mark_fonts]
    });
    lists.flatten().cloned().collect()
}

#[test]
fn each_family_a_theme_bundles_maps_to_files_that_exist_and_name_it() {
    let table = FontFiles::shipped().expect("the font table loads");
    let mut checked = 0;
    for family in named()
        .iter()
        .filter(|f| !SYSTEM_FONTS.contains(&f.as_str()))
    {
        let files = table
            .files(family)
            .unwrap_or_else(|| panic!("{family} has no files"));
        assert!(!files.is_empty(), "{family} lists no file");
        for rel in files {
            let path = assets().join(kx_theme::assets::FONT_DIR).join(rel);
            let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{rel}: {e}"));
            assert!(
                fontname::families(&bytes).contains(family),
                "{rel} is not {family}"
            );
        }
        checked += 1;
    }
    assert!(checked > 0, "the check reached the bundled families");
}

#[test]
fn every_bundled_file_belongs_to_a_family_some_theme_names() {
    let table = FontFiles::shipped().expect("the font table loads");
    let named = named();
    let dir = assets().join(kx_theme::assets::FONT_DIR);
    let listed: BTreeSet<_> = table
        .families()
        .inspect(|f| assert!(named.contains(*f), "no theme names {f}"))
        .flat_map(|f| table.files(f).unwrap_or_default())
        .map(|rel| dir.join(rel))
        .collect();
    let files: BTreeSet<_> = font_files(&assets())
        .expect("assets/fonts reads")
        .into_iter()
        .collect();
    assert_eq!(
        listed, files,
        "the table lists every bundled file, and only those"
    );
}

#[test]
fn the_table_gives_each_file_the_name_the_bundle_gives_it() {
    let table = FontFiles::shipped().expect("the font table loads");
    let b = bundle(&assets()).expect("the bundle is found");
    let bundled: BTreeSet<&str> = b.fonts.iter().map(|f| f.name.as_str()).collect();
    let names: BTreeSet<String> = table
        .families()
        .flat_map(|f| table.names(f).unwrap_or_default())
        .collect();
    assert_eq!(
        names.iter().map(String::as_str).collect::<BTreeSet<_>>(),
        bundled
    );
    assert!(
        table.names("Segoe UI").is_none(),
        "a Windows family is not bundled"
    );
}
