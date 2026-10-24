//! The manifest: the project's own `words.toml`, and the sources and languages a manifest refuses.

use super::super::fixture::{self, ABC_SHA256, abc_source};
use super::super::manifest::{Manifest, Origin};
use super::super::{MANIFEST, WordsError};

#[test]
fn the_project_manifest_parses_with_every_language_file() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    for (iso, max) in [("en", 150_000), ("fr", 150_000), ("ar", 250_000)] {
        let plan = manifest.plan(iso).unwrap();
        assert_eq!(plan.list, format!("wordfreq/large_{iso}.msgpack.gz"));
        assert_eq!(plan.max_words, max);
        assert!(plan.dict.is_some(), "{iso} has a dictionary");
    }
    assert_eq!(manifest.plan("fr").unwrap().elisions.len(), 9);
    assert!(manifest.plan("ar").unwrap().hamza.is_some());
    let de = manifest.plan("de").unwrap();
    assert_eq!(
        (de.list.as_str(), de.max_words),
        ("wordfreq/small_de.msgpack.gz", 60_000)
    );
    assert!(de.dict.is_none() && de.hamza.is_none());
}

#[test]
fn the_project_manifest_gives_each_source_its_origin() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    let wordfreq = "https://raw.githubusercontent.com/rspeer/wordfreq/912caf64b657478d1dff1138efdc078947d54bb1";
    let url = |rel: &str| manifest.origin(rel);
    assert_eq!(
        url("wordfreq/large_en.msgpack.gz"),
        Some(Origin::Url(format!(
            "{wordfreq}/wordfreq/data/large_en.msgpack.gz"
        )))
    );
    assert_eq!(
        url("wordfreq/README.md"),
        Some(Origin::Url(format!("{wordfreq}/README.md")))
    );
    let zip = "dict/en/hunspell-en_US-2026.02.25.zip";
    assert_eq!(url("dict/en/unzipped/en_US.aff"), Some(Origin::Zip(zip)));
    assert_eq!(url("nope"), None);
    assert_eq!(manifest.sources.len(), 66);
}

#[test]
fn an_unknown_language_has_no_plan() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    for iso in ["xx", "", "de/../en", "{iso}"] {
        assert_eq!(
            manifest.plan(iso).err(),
            Some(WordsError::Lang(iso.to_string()))
        );
    }
}

#[test]
fn the_fixture_manifest_parses() {
    assert!(fixture::manifest_with(&abc_source()).is_ok());
}

#[test]
fn a_bad_source_is_refused() {
    let line = |rest: &str| format!(r#""d/x.txt" = {{ {rest}, size = 3 }}"#);
    let hash = format!(r#"sha256 = "{ABC_SHA256}""#);
    let bad = [
        line(&format!(r#"base = "nope", {hash}"#)),
        line(r#"base = "test", sha256 = "abc""#),
        line(&format!(
            r#"base = "test", sha256 = "{}""#,
            ABC_SHA256.to_uppercase()
        )),
        line(&format!(r#"base = "test", zip = "d/fr.aff", {hash}"#)),
        line(&hash),
        line(&format!(r#"dir = "x/", zip = "d/fr.aff", {hash}"#)),
        line(&format!(r#"zip = "d/x.txt", {hash}"#)),
        format!(r#""../x.txt" = {{ base = "test", {hash}, size = 3 }}"#),
        line(&format!(r#"base = "test", {hash}, extra = 1"#)),
    ];
    for source in bad {
        let result = fixture::manifest_with(&source);
        assert!(
            matches!(result, Err(WordsError::Manifest(_))),
            "{source} was accepted"
        );
    }
}

#[test]
fn a_bad_language_is_refused() {
    let text = fixture::manifest_text("");
    let swaps = [
        (r#"dic = "d/fr.dic""#, r#"dic = "d/nope.dic""#),
        ("max_words = 5", "max_words = 0"),
        ("[lang.fr]", "[lang.toolongcode]"),
        ("small_{iso}", "small_de"),
        ("max_words = 2", "max_words = 0"),
        (
            "[lang.fr]",
            "[lang.fr]\nhamza = { bare = \"ا\", marked = [\"ا\"] }",
        ),
    ];
    for (from, to) in swaps {
        assert!(text.contains(from), "{from}");
        let result = Manifest::parse(&text.replacen(from, to, 1));
        assert!(
            matches!(result, Err(WordsError::Manifest(_))),
            "{to} was accepted"
        );
    }
}
