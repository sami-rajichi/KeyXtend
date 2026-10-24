//! `words build` end to end on tiny sources in a temp folder: the packs open with `kx_mod_predict`'s reader, the
//! chains follow the text, held-back sentences stay out, and the credits name every source.

use std::fmt::Write as _;
use std::path::Path;

use kx_mod_predict::Pack;
use sha2::{Digest, Sha256};

use super::super::build::{self, Paths};
use super::super::fixture::text::{tatoeba, wiki};
use super::super::fixture::{self, AFF, dic};
use super::super::manifest::Manifest;
use super::super::tatoeba::held;
use super::super::{WordsError, credits};
use crate::devtools::paths::join_rel;
use crate::test_support::TempDir;

/// Bucket of the fixture's most frequent word; the others follow one by one.
const TOP: usize = 200;

/// A wordfreq list holding `words` from bucket `TOP` on.
fn list(words: &[&str]) -> Vec<u8> {
    let mut buckets: Vec<&[&str]> = vec![&[]; TOP];
    buckets.extend(words.iter().map(std::slice::from_ref));
    fixture::list(&buckets)
}

/// Writes each `(rel, bytes)` under `src` and returns its `[sources]` lines.
fn sources(src: &Path, files: &[(&str, Vec<u8>)]) -> String {
    let mut lines = String::new();
    for (rel, bytes) in files {
        let path = join_rel(src.to_path_buf(), rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        let hex = Sha256::digest(bytes)
            .iter()
            .fold(String::new(), |mut hex, b| {
                write!(hex, "{b:02x}").unwrap();
                hex
            });
        let size = bytes.len();
        writeln!(
            lines,
            r#""{rel}" = {{ base = "test", sha256 = "{hex}", size = {size} }}"#
        )
        .unwrap();
    }
    lines
}

/// Four ids Tatoeba counts, then one it holds back.
fn ids() -> (Vec<String>, String) {
    let all = (1..).map(|i: u32| i.to_string());
    let counted = all.clone().filter(|id| !held(id, 20)).take(4).collect();
    (counted, all.clone().find(|id| held(id, 20)).unwrap())
}

/// The fixture manifest with real French files, French text, a German small list and credit sections, written
/// under `src`.
fn manifest(src: &Path) -> Manifest {
    let lines = sources(src, &files());
    let text = fixture::manifest_text("")
        .lines()
        .filter(|l| !l.starts_with("\"w/large_fr.gz\"") && !l.starts_with("\"d/fr."))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("max_words = 5", "max_words = 50")
        .replace(
            r#"file = "d/fr.aff", from = "A", to = "B""#,
            r###"file = "w/README.md", from = "## License", to = "## ""###,
        )
        .replace(
            r#"file = "d/fr.dic", from = "A", to = "B""#,
            r#"file = "d/README.txt", from = "COPYRIGHT", to = "Build""#,
        );
    let lang = r#"
[lang.fr.text]
chain_budget_kb = 10
tatoeba = { file = "t/fra.tsv.bz2", max_sentences = 100 }
wiki = { files = ["k/frwiki.json.bz2"], max_bytes = 1000 }
"#;
    Manifest::parse(&format!("{text}\n{lines}{lang}")).unwrap()
}

/// The fixture's source files: French words, dictionary, Tatoeba and Wikipedia text, a German small list and the
/// two files the credits copy from.
fn files() -> Vec<(&'static str, Vec<u8>)> {
    let words = ["le", "chat", "dort", "l", "homme", "est", "ici"];
    let (n, held_id) = ids();
    let sentences = [
        (n[0].as_str(), "Le chat dort.", "ann"),
        (n[1].as_str(), "L'homme est ici.", "bob"),
        (n[2].as_str(), "Le chat est ici.", "ann"),
        (held_id.as_str(), "Le chat dort ici.", "cy"),
        (n[3].as_str(), "Le chat dort.", "-"),
    ];
    vec![
        ("w/large_fr.gz", list(&words)),
        ("d/fr.aff", AFF.as_bytes().to_vec()),
        (
            "d/fr.dic",
            dic(&["le", "chat", "dort", "l", "l'", "homme", "est", "ici"]).into_bytes(),
        ),
        ("w/small_de.gz", list(&["der", "die"])),
        ("t/fra.tsv.bz2", tatoeba(&sentences)),
        (
            "k/frwiki.json.bz2",
            wiki(&[
                Some("Le chat dort. L'homme dort."),
                Some("Le chat est ici."),
            ]),
        ),
        (
            "w/README.md",
            b"# wordfreq\n\n## License\n\nShared under CC BY-SA.\n\n## Next\n".to_vec(),
        ),
        (
            "d/README.txt",
            b"COPYRIGHT, SOURCES\nCopyright 2000 by Kevin.\nBuild Date: x\n".to_vec(),
        ),
    ]
}

/// Builds the fixture into a temp folder and returns it with the lines said.
fn built() -> (TempDir, Vec<String>) {
    let dir = TempDir::new("words-build");
    let src = dir.path().join("src");
    let manifest = manifest(&src);
    let paths = Paths {
        src,
        out: dir.path().join("out"),
        held: dir.path().join("held"),
    };
    let mut said = Vec::new();
    build::run(&manifest, &paths, &mut |l| said.push(l.to_string())).unwrap();
    (dir, said)
}

/// The followers of the words `context` in `pack`, best first.
fn next<'a>(pack: &Pack<'a>, context: &[&str]) -> Vec<&'a str> {
    let ids: Vec<u32> = context.iter().map(|w| pack.run(w).start).collect();
    pack.followers(&ids)
        .map(|(id, _)| pack.spelling(id))
        .collect()
}

#[test]
fn the_built_pack_opens_and_its_chains_follow_the_text() {
    let (dir, said) = built();
    let bytes = std::fs::read(dir.path().join("out").join("fr.kxw")).unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    assert!(pack.flags().checked && pack.flags().chains);
    assert_eq!(next(&pack, &["le"]), ["chat"]);
    assert_eq!(
        next(&pack, &["chat"]),
        ["dort", "est"],
        "9 against 5, Tatoeba weighing 4"
    );
    assert_eq!(next(&pack, &["le", "chat"]), ["dort", "est"]);
    assert_eq!(
        next(&pack, &["l", "homme"]),
        ["est", "dort"],
        "L' splits off"
    );
    let scores: Vec<u8> = pack
        .followers(&[pack.run("chat").start])
        .map(|(_, s)| s)
        .collect();
    assert_eq!(scores, [250, 244], "9 and 5 of the 14 times chat was seen");
    assert!(
        said.iter().any(|l| l.starts_with("pack fr: 7 words, ")),
        "{said:?}"
    );
}

#[test]
fn held_back_sentences_go_to_their_file_and_are_never_counted() {
    let (dir, _) = built();
    let held = std::fs::read_to_string(dir.path().join("held").join("fr.txt")).unwrap();
    assert_eq!(held, "Le chat dort ici.\n");
    let bytes = std::fs::read(dir.path().join("out").join("fr.kxw")).unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    assert!(
        next(&pack, &["dort"]).is_empty(),
        "dort ici is only held back"
    );
}

#[test]
fn a_language_without_text_gets_a_pack_without_chains() {
    let (dir, said) = built();
    let bytes = std::fs::read(dir.path().join("out").join("de.kxw")).unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    assert!(!pack.flags().chains && !pack.flags().checked);
    assert_eq!(pack.word_count(), 2);
    assert!(
        said.iter()
            .any(|l| l.starts_with("pack de: 2 words, 0 contexts")),
        "{said:?}"
    );
}

#[test]
fn the_credits_copy_the_licence_sections_and_name_every_source() {
    let (dir, _) = built();
    let out = dir.path().join("out");
    let read = |name: &str| std::fs::read_to_string(out.join(name)).unwrap();
    let credits = read("CREDITS.txt");
    for line in [
        "## License",
        "Shared under CC BY-SA.",
        "Copyright 2000 by Kevin.",
    ] {
        assert!(credits.contains(line), "{line} missing");
    }
    assert!(!credits.contains("## Next") && !credits.contains("Build Date"));
    for rel in [
        "w/README.md",
        "w/large_fr.gz",
        "w/small_de.gz",
        "t/fra.tsv.bz2",
        "k/frwiki.json.bz2",
        "d/fr.aff",
        "d/fr.dic",
    ] {
        let name = rel.rsplit('/').next().unwrap();
        assert!(
            credits.contains(&format!("https://example.invalid/data/{name}")),
            "{rel} missing"
        );
    }
    assert!(!credits.contains('{'), "every placeholder is filled");
    assert_eq!(
        read("AUTHORS.txt"),
        "ann\nbob\n",
        "cy is held back, - is no one"
    );
    assert_eq!(read("LICENSE.txt"), "Shared alike.\nSee the credits.\n");
}

#[test]
fn the_template_fills_known_placeholders_only() {
    let values = std::collections::BTreeMap::from([("a", "x{b}".to_string())]);
    assert_eq!(
        credits::fill("1 {a} 2", &values).unwrap(),
        "1 x{b} 2",
        "a value is not filled again"
    );
    for bad in ["{b}", "{a"] {
        assert!(
            matches!(credits::fill(bad, &values), Err(WordsError::Manifest(_))),
            "{bad}"
        );
    }
}
