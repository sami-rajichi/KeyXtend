//! What `words build` writes and checks: every source before any language, the download budget, the packs a past
//! run left, and the credits with their authors.

use super::super::build::{self, Paths};
use super::super::manifest::Section;
use super::super::{WordsError, credits, out};
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
        "## Citations",
        "- SUBTLEX-UK.",
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

#[test]
fn a_stray_pack_in_another_case_leaves_the_fresh_one_written_and_measured() {
    let dir = TempDir::new("words-build");
    let (result, said) = run_with(&dir, |p| {
        std::fs::create_dir_all(&p.out).unwrap();
        std::fs::write(p.out.join("FR.KXW"), vec![7_u8; 5_000]).unwrap();
    });
    result.unwrap();
    let out = paths(&dir).out;
    let fresh = std::fs::read(out.join("fr.kxw")).unwrap();
    assert!(kx_mod_predict::Pack::parse(&fresh).is_ok(), "{said:?}");
    let names = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name());
    let fr = names.filter(|n| n.eq_ignore_ascii_case("fr.kxw")).count();
    let download = said.last().unwrap();
    assert!(
        fr == 1 && download.starts_with("download: 5 files, "),
        "{said:?}"
    );
}

#[test]
fn a_name_in_another_case_is_the_written_pack_unless_both_are_listed() {
    let listed = |names: &[&str]| names.iter().map(ToString::to_string).collect::<Vec<_>>();
    let one = listed(&["FR.kxw", "xx.kxw"]);
    assert_eq!(
        out::stale(&one, &["fr.kxw"]),
        ["xx.kxw"],
        "a folder that ignores case kept the old name"
    );
    let both = listed(&["FR.kxw", "fr.kxw"]);
    assert_eq!(
        out::stale(&both, &["fr.kxw"]),
        ["FR.kxw"],
        "a folder that minds case holds two files"
    );
}

#[test]
fn a_section_without_an_end_runs_to_the_files_end_and_a_missing_mark_is_refused() {
    let dir = TempDir::new("words-copy");
    let path = dir.path().join("README.md");
    std::fs::write(
        &path,
        "# w\n## Cite\n- SUBTLEX-UK.\n  <https://x.invalid>\n\n",
    )
    .unwrap();
    let section = |from: &str, to: Option<&str>| Section {
        file: String::new(),
        from: from.to_string(),
        to: to.map(str::to_string),
    };
    let copy = |s: &Section| credits::copy(&path, s);
    let cite = "## Cite\n- SUBTLEX-UK.\n  <https://x.invalid>";
    assert_eq!(copy(&section("## Cite", None)).unwrap(), cite);
    assert_eq!(copy(&section("# w", Some("## "))).unwrap(), "# w");
    for missing in [section("## Nope", None), section("# w", Some("## Nope"))] {
        let refused = copy(&missing);
        assert!(
            matches!(&refused, Err(WordsError::Io(why)) if why.ends_with("\"## Nope\"")),
            "{refused:?}"
        );
    }
}

#[test]
fn each_source_is_checked_once_though_two_credit_sections_share_it() {
    let dir = TempDir::new("words-build");
    let all = credits::sources(&manifest(&paths(&dir).src, NO_SKEW));
    let readme = all.iter().filter(|f| *f == "w/README.md").count();
    assert_eq!(readme, 1, "{all:?}");
}
