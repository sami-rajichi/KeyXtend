//! The pack index: the build writes it beside the packs it wrote, with the layout codes the manifest's aliases serve,
//! and `words index` rewrites it from a folder's packs, each of which must parse and carry the language its file names.

use std::collections::BTreeMap;
use std::path::Path;

use kx_mod_predict::{Flags, PackWriter};
use kx_module_api::ports::{IndexError, PACK_EXT, PACK_INDEX, PackIndex, pack_file};

use super::super::fixture;
use super::super::index;
use super::super::manifest::Manifest;
use super::super::{MANIFEST, WordsError};
use super::build::built;
use crate::test_support::TempDir;

/// A one-word pack of language `iso`.
fn pack(iso: &str) -> Vec<u8> {
    let mut writer = PackWriter::new(iso, Flags::default(), 1).unwrap();
    writer.add_word("abc", 1).unwrap();
    writer.finish().unwrap()
}

/// The index in `dir`.
fn index_in(dir: &Path) -> PackIndex {
    PackIndex::read(&std::fs::read(dir.join(PACK_INDEX)).unwrap()).unwrap()
}

/// The languages the index in `dir` lists.
fn listed(dir: &Path) -> Vec<String> {
    index_in(dir).packs().map(str::to_owned).collect()
}

/// `pack` serving the layout codes `codes`.
fn aliases(pack: &str, codes: &[&str]) -> BTreeMap<String, Vec<String>> {
    let codes = codes.iter().map(|c| (*c).to_owned()).collect();
    BTreeMap::from([(pack.to_owned(), codes)])
}

#[test]
fn the_build_lists_every_pack_it_wrote() {
    let (dir, _) = built();
    let out = dir.path().join("out");
    let isos = listed(&out);
    assert!(!isos.is_empty());
    for iso in &isos {
        assert!(out.join(pack_file(iso).unwrap()).is_file(), "{iso}");
    }
    let packs = std::fs::read_dir(&out).unwrap().filter_map(|e| {
        let path = e.unwrap().path();
        path.extension()
            .is_some_and(|x| x == PACK_EXT)
            .then_some(path)
    });
    assert_eq!(packs.count(), isos.len(), "one line per pack");
}

#[test]
fn the_build_writes_each_alias_after_its_pack() {
    let (dir, _) = built();
    let index = index_in(&dir.path().join("out"));
    assert_eq!(index.pack_of("wa"), Some("fr"), "the fixture's alias");
}

#[test]
fn the_project_serves_croatian_and_bosnian_from_serbo_croatian_and_never_serbian() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    assert_eq!(manifest.packs.aliases, aliases("sh", &["hr", "bs"]));
    let isos = manifest.isos();
    let isos: Vec<&str> = isos.iter().map(String::as_str).collect();
    let index = PackIndex::new(&isos, &manifest.packs.aliases).unwrap();
    assert_eq!(
        (index.pack_of("hr"), index.pack_of("bs")),
        (Some("sh"), Some("sh"))
    );
    assert_eq!(
        index.pack_of("sr"),
        None,
        "no script tag tells Latin Serbian"
    );
}

#[test]
fn an_alias_of_no_pack_a_bad_code_or_a_pack_code_is_refused() {
    let text = fixture::manifest_text("");
    let from = r#"fr = ["wa"]"#;
    assert!(text.contains(from));
    for to in [r#"xx = ["wa"]"#, r#"fr = ["WA"]"#, r#"fr = ["fr"]"#] {
        let result = Manifest::parse(&text.replacen(from, to, 1));
        assert!(
            matches!(result, Err(WordsError::Manifest(_))),
            "{to} was accepted"
        );
    }
}

#[test]
fn the_index_command_lists_the_packs_in_a_folder_and_the_codes_they_serve() {
    let dir = TempDir::new("words-index");
    for iso in ["fr", "en", "fil"] {
        std::fs::write(dir.path().join(pack_file(iso).unwrap()), pack(iso)).unwrap();
    }
    let mut said = Vec::new();
    index::run(dir.path(), &aliases("fil", &["tl"]), &mut |l| {
        said.push(l.to_owned());
    })
    .unwrap();
    assert_eq!(listed(dir.path()), ["en", "fil", "fr"]);
    assert_eq!(index_in(dir.path()).pack_of("tl"), Some("fil"));
    assert_eq!(said.len(), 1, "{said:?}");
}

#[test]
fn a_folder_without_packs_writes_no_index() {
    let dir = TempDir::new("words-index-empty");
    let result = index::run(dir.path(), &BTreeMap::new(), &mut |_| {});
    let empty = IndexError::Empty.to_string();
    assert!(
        matches!(&result, Err(WordsError::Pack(t)) if t.contains(&empty)),
        "the index's own rule refuses it: {result:?}"
    );
    assert!(!dir.path().join(PACK_INDEX).exists());
}

#[test]
fn a_pack_of_another_language_or_a_damaged_one_writes_no_index() {
    for (name, bytes) in [("en", pack("fr")), ("en", b"not a pack".to_vec())] {
        let dir = TempDir::new("words-index-bad");
        std::fs::write(dir.path().join(pack_file(name).unwrap()), bytes).unwrap();
        assert!(index::run(dir.path(), &BTreeMap::new(), &mut |_| {}).is_err());
        assert!(!dir.path().join(PACK_INDEX).exists());
    }
}
