//! What `words build` writes and checks: every source before any language, the download budget, the packs a past
//! run left, and the credits with their authors.

use super::super::WordsError;
use super::super::build::{self, Paths};
use super::build::{NO_SKEW, built, manifest, paths};
use crate::test_support::TempDir;

/// Builds the fixture under `dir` after `prepare` touches its folders, with the result and the lines said.
fn run_with(dir: &TempDir, prepare: impl FnOnce(&Paths)) -> (Result<(), WordsError>, Vec<String>) {
    let paths = paths(dir);
    let manifest = manifest(&paths.src, NO_SKEW);
    prepare(&paths);
    let mut said = Vec::new();
    let result = build::run(&manifest, &paths, &mut |l| said.push(l.to_string()));
    (result, said)
}

#[test]
fn a_missing_source_fails_before_any_language_is_built() {
    let dir = TempDir::new("words-build");
    let (result, said) = run_with(&dir, |p| {
        std::fs::remove_file(p.src.join("t").join("fra.tsv.bz2")).unwrap();
    });
    let Err(WordsError::Bad(files)) = result else {
        panic!("{result:?}");
    };
    assert!(
        files.iter().any(|f| f.starts_with("t/fra.tsv.bz2")),
        "{files:?}"
    );
    assert!(said.is_empty() && !paths(&dir).out.exists(), "{said:?}");
}

#[test]
fn a_build_over_the_download_budget_writes_nothing() {
    let dir = TempDir::new("words-build");
    let paths = paths(&dir);
    let mut manifest = manifest(&paths.src, NO_SKEW);
    manifest.packs.download_kb = 1;
    let refused = build::run(&manifest, &paths, &mut |_| {});
    assert!(matches!(refused, Err(WordsError::Pack(why)) if why.ends_with("the build is refused")));
    assert!(!paths.out.exists());
}

#[test]
fn a_pack_this_run_did_not_build_goes_and_the_folder_is_measured() {
    let dir = TempDir::new("words-build");
    let (result, said) = run_with(&dir, |p| {
        std::fs::create_dir_all(&p.out).unwrap();
        std::fs::write(p.out.join("xx.kxw"), vec![7_u8; 5_000]).unwrap();
        std::fs::write(p.out.join("notes.txt"), "kept").unwrap();
    });
    result.unwrap();
    let out = paths(&dir).out;
    assert!(!out.join("xx.kxw").exists() && out.join("notes.txt").exists());
    let stale = "stale: removed xx.kxw, which this run did not build";
    assert!(said.iter().any(|l| l == stale), "{said:?}");
    let download = said.last().unwrap();
    assert!(download.starts_with("download: 5 files, "), "{download}");
}

#[test]
fn the_credits_copy_the_licence_sections_and_name_every_source() {
    let (dir, _) = built();
    let out = dir.path().join("out");
    let read = |name: &str| std::fs::read_to_string(out.join(name)).unwrap();
    let credits = read("CREDITS.txt");
    let lines = [
        "## License",
        "Shared under CC BY-SA.",
        "Copyright 2000 by Kevin.",
        "language editions fr, from the Wikimedia search-index dump of data, in these files:",
    ];
    for line in lines {
        assert!(credits.contains(line), "{line} missing");
    }
    assert!(!credits.contains("## Next") && !credits.contains("Build Date"));
    let rels = [
        "w/README.md",
        "w/large_fr.gz",
        "w/small_de.gz",
        "t/fra.tsv.bz2",
    ];
    for rel in rels
        .iter()
        .chain(&["k/frwiki.json.bz2", "d/fr.aff", "d/fr.dic"])
    {
        let name = rel.rsplit('/').next().unwrap();
        let url = format!("https://example.invalid/data/{name}");
        assert!(credits.contains(&url), "{rel} missing");
    }
    assert!(!credits.contains('{'), "every placeholder is filled");
    let authors = read("AUTHORS.txt");
    assert_eq!(
        authors, "ann\nbob\n",
        "cy is held back, - is no one, dee only repeats ann"
    );
    assert_eq!(read("LICENSE.txt"), "Shared alike.\nSee the credits.\n");
}
