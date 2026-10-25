//! The pack index: the build writes it beside the packs it wrote, and `words index` rewrites it from a folder's packs,
//! each of which must parse and carry the language its file names.

use std::path::Path;

use kx_mod_predict::{Flags, PackWriter};
use kx_module_api::ports::{PACK_EXT, PACK_INDEX, pack_file, read_index};

use super::super::index;
use super::build::built;
use crate::test_support::TempDir;

/// A one-word pack of language `iso`.
fn pack(iso: &str) -> Vec<u8> {
    let mut writer = PackWriter::new(iso, Flags::default(), 1).unwrap();
    writer.add_word("abc", 1).unwrap();
    writer.finish().unwrap()
}

/// The languages the index in `dir` lists.
fn listed(dir: &Path) -> Vec<String> {
    let bytes = std::fs::read(dir.join(PACK_INDEX)).unwrap();
    read_index(&bytes).unwrap().into_iter().collect()
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
fn the_index_command_lists_the_packs_in_a_folder() {
    let dir = TempDir::new("words-index");
    for iso in ["fr", "en", "fil"] {
        std::fs::write(dir.path().join(pack_file(iso).unwrap()), pack(iso)).unwrap();
    }
    let mut said = Vec::new();
    index::run(dir.path(), &mut |l| said.push(l.to_owned())).unwrap();
    assert_eq!(listed(dir.path()), ["en", "fil", "fr"]);
    assert_eq!(said.len(), 1, "{said:?}");
}

#[test]
fn a_pack_of_another_language_or_a_damaged_one_writes_no_index() {
    for (name, bytes) in [("en", pack("fr")), ("en", b"not a pack".to_vec())] {
        let dir = TempDir::new("words-index-bad");
        std::fs::write(dir.path().join(pack_file(name).unwrap()), bytes).unwrap();
        assert!(index::run(dir.path(), &mut |_| {}).is_err());
        assert!(!dir.path().join(PACK_INDEX).exists());
    }
}
