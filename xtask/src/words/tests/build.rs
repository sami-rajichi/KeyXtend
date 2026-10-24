//! `words build` end to end on tiny sources in a temp folder: the packs open with `kx_mod_predict`'s reader, the
//! chains follow the text, and held-back sentences stay out.

use std::fmt::Write as _;
use std::path::Path;

use kx_mod_predict::Pack;
use sha2::{Digest, Sha256};

use super::super::build::{self, Paths};
use super::super::fixture::text::{tatoeba, wiki};
use super::super::fixture::{self, AFF, dic};
use super::super::manifest::Manifest;
use super::super::tatoeba::held;
use crate::devtools::paths::join_rel;
use crate::test_support::TempDir;

/// Bucket of the fixture's most frequent word; the others follow one by one.
const TOP: usize = 200;
/// A skew ratio no word of the fixture's Tatoeba reaches.
pub(super) const NO_SKEW: &str = "1000.0";

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
/// under `src`, Tatoeba's words scaled past `skew` times their wordfreq share.
pub(super) fn manifest(src: &Path, skew: &str) -> Manifest {
    let lines = sources(src, &files());
    let text = fixture::manifest_text("")
        .lines()
        .filter(|l| !l.starts_with("\"w/large_fr.gz\"") && !l.starts_with("\"d/fr."))
        .collect::<Vec<_>>()
        .join("\n")
        .replace("max_words = 5", "max_words = 50")
        .replace("skew_ratio = 10.0", &format!("skew_ratio = {skew}"))
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
        (n[2].as_str(), "Le chat est ici.", "-"),
        (held_id.as_str(), "Le chat dort ici.", "cy"),
        (n[3].as_str(), "Le chat dort.", "dee"),
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
            wiki(&[Some("Le chat dort. L'homme dort."), Some("Le chat est là.")]),
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

/// The fixture's folders under `dir`.
pub(super) fn paths(dir: &TempDir) -> Paths {
    let at = |name: &str| dir.path().join(name);
    Paths {
        src: at("src"),
        out: at("out"),
        held: at("held"),
    }
}

/// Builds the fixture into a temp folder, Tatoeba scaled past `skew`, and returns it with the lines said.
fn built_with(skew: &str) -> (TempDir, Vec<String>) {
    let dir = TempDir::new("words-build");
    let paths = paths(&dir);
    let manifest = manifest(&paths.src, skew);
    let mut said = Vec::new();
    build::run(&manifest, &paths, &mut |l| said.push(l.to_string())).unwrap();
    (dir, said)
}

/// Builds the fixture with no Tatoeba word scaled.
pub(super) fn built() -> (TempDir, Vec<String>) {
    built_with(NO_SKEW)
}

/// The scores of the followers of `word` in `pack`.
fn scores(pack: &Pack<'_>, word: &str) -> Vec<u8> {
    pack.followers(&[pack.run(word).start])
        .map(|(_, s)| s)
        .collect()
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
        ["est", "dort"],
        "5 against 4: Tatoeba weighs 4, and its second Le chat dort and Wikipedia's are repeats"
    );
    assert_eq!(next(&pack, &["le", "chat"]), ["est", "dort"]);
    assert_eq!(
        next(&pack, &["l'", "homme"]),
        ["est", "dort"],
        "L' splits off"
    );
    assert_eq!(
        scores(&pack, "chat"),
        [249, 246],
        "5 and 4 of the 9 times chat was seen"
    );
    for start in [
        "pack fr: 7 words, ",
        "repeats fr: 2 of 7 sentences skipped as repeats, 5 remembered",
    ] {
        assert!(said.iter().any(|l| l.starts_with(start)), "{said:?}");
    }
}

#[test]
fn tatoebas_overused_words_end_fewer_chains_and_wikipedia_is_not_scaled() {
    let (dir, said) = built_with("15.0");
    let bytes = std::fs::read(dir.path().join("out").join("fr.kxw")).unwrap();
    let pack = Pack::parse(&bytes).unwrap();
    let skew = "skew fr: 4 words; first 2: ici 21, est 20";
    assert!(said.iter().any(|l| l == skew), "{said:?}");
    assert_eq!(
        next(&pack, &["chat"]),
        ["dort", "est"],
        "est takes 2 of 11 Tatoeba tokens, 20 times its share: 4 × 15 / 20.4 + 1 of Wikipedia"
    );
    assert_eq!(scores(&pack, "chat"), [246, 246], "4 and 3.94 of 9");
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
