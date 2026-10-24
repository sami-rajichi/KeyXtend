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
    let de = manifest.plan("de").unwrap();
    assert_eq!(
        (de.list.as_str(), de.max_words),
        ("wordfreq/small_de.msgpack.gz", 60_000)
    );
    assert!(de.dict.is_none());
}

#[test]
fn the_project_manifest_hardens_curl() {
    let fetch = Manifest::parse(MANIFEST).unwrap().fetch;
    assert_eq!(
        fetch.args.first().map(String::as_str),
        Some("-q"),
        "no curl config file is read"
    );
    let has = |flag: &str| {
        fetch
            .args
            .windows(2)
            .any(|w| w[0] == flag && w[1] == "=https")
    };
    assert!(has("--proto") && has("--proto-redir"), "{:?}", fetch.args);
    let flags = [&fetch.size_flag, &fetch.out_flag, &fetch.url_flag];
    assert_eq!(flags, ["--max-filesize", "--output", "--url"]);
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
        ("max_words = 5", "max_words = 5\nelisions = [\"l\"]"),
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

#[test]
fn a_rule_threshold_out_of_range_is_refused() {
    let text = fixture::manifest_text("");
    let swaps = [
        ("max_gap_cb = 100", "max_gap_cb = 0"),
        ("max_gap_cb = 100", "max_gap_cb = 901"),
        ("main_script_share = 0.9", "main_script_share = 0.5"),
        ("main_script_share = 0.9", "main_script_share = 1.1"),
        ("min_case_seen = 20", "min_case_seen = 0"),
        ("piece_ratio = 20.0", "piece_ratio = 1.0"),
        ("piece_ratio = 20.0", "piece_ratio = inf"),
        ("piece_min_expected = 100.0", "piece_min_expected = 0.0"),
        ("piece_min_expected = 100.0", "piece_min_expected = nan"),
        ("coverage_top = 3", "coverage_top = 0"),
        ("show = 2", "show = 0"),
        (r#"part = "part""#, r#"part = """#),
        (r#"part = "part""#, r#"part = "a/b""#),
        ("https://example.invalid", "http://example.invalid"),
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

#[test]
fn the_edges_of_each_range_are_accepted() {
    let text = fixture::manifest_text("");
    let swaps = [
        ("max_gap_cb = 100", "max_gap_cb = 900"),
        ("main_script_share = 0.9", "main_script_share = 1.0"),
        ("min_case_seen = 20", "min_case_seen = 1"),
    ];
    for (from, to) in swaps {
        let result = Manifest::parse(&text.replacen(from, to, 1));
        assert!(result.is_ok(), "{to} was refused: {result:?}");
    }
}
