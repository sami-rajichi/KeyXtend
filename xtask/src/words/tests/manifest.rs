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

/// Swaps that each put one rule, text or credits value out of its range.
const OUT_OF_RANGE: &[(&str, &str)] = &[
    ("max_gap_cb = 100", "max_gap_cb = 0"),
    ("max_gap_cb = 100", "max_gap_cb = 901"),
    ("main_script_share = 0.9", "main_script_share = 0.5"),
    ("main_script_share = 0.9", "main_script_share = 1.1"),
    ("min_case_seen = 20", "min_case_seen = 0"),
    ("twin_ratio = 10.0", "twin_ratio = 0.5"),
    ("twin_ratio = 10.0", "twin_ratio = nan"),
    ("piece_ratio = 20.0", "piece_ratio = 1.0"),
    ("piece_ratio = 20.0", "piece_ratio = inf"),
    ("piece_min_seen = 100", "piece_min_seen = 0"),
    ("coverage_top = 3", "coverage_top = 0"),
    ("show = 2", "show = 0"),
    (r#"part = "part""#, r#"part = """#),
    (r#"part = "part""#, r#"part = "a/b""#),
    ("https://example.invalid", "http://example.invalid"),
    ("min_chain = 2", "min_chain = 1"),
    ("max_chain = 3", "max_chain = 6"),
    ("max_chain = 3", "max_chain = 1"),
    ("tatoeba_weight = 4", "tatoeba_weight = 0"),
    ("held_back_per_mille = 20", "held_back_per_mille = 1000"),
    ("max_live_chains = 100", "max_live_chains = 0"),
    ("followers = 2", "followers = 0"),
    (r#"null = "-""#, r#"null = """#),
    (r#"held = "held""#, r#"held = "../held""#),
    (r#"out = "out""#, r#"out = """#),
    (r#"licence = "LICENSE.txt""#, r#"licence = "a/b.txt""#),
    (r#"["Shared alike.", "See the credits."]"#, "[]"),
    (r#"file = "d/fr.aff""#, r#"file = "d/nope.aff""#),
    (r#"to = "B" }"#, r#"to = "" }"#),
    ("sentence_ends = [\".\", \"?\"]", "sentence_ends = []"),
];

#[test]
fn a_rule_threshold_out_of_range_is_refused() {
    let text = fixture::manifest_text("");
    for &(from, to) in OUT_OF_RANGE {
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
        ("twin_ratio = 10.0", "twin_ratio = 1.0"),
        ("max_chain = 3", "max_chain = 2"),
        ("max_chain = 3", "max_chain = 5"),
        ("held_back_per_mille = 20", "held_back_per_mille = 999"),
    ];
    for (from, to) in swaps {
        let result = Manifest::parse(&text.replacen(from, to, 1));
        assert!(result.is_ok(), "{to} was refused: {result:?}");
    }
}

#[test]
fn the_project_manifest_gives_en_fr_and_ar_their_text_and_budget() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    for (iso, kb) in [("en", 3000), ("fr", 3000), ("ar", 4000)] {
        let text = manifest.plan(iso).unwrap().text.unwrap();
        assert_eq!(text.chain_budget_kb, kb, "{iso}");
        assert!(text.tatoeba.is_some() && text.wiki.is_some(), "{iso}");
    }
    assert!(manifest.plan("de").unwrap().text.is_none());
    let t = &manifest.text;
    let values = (
        t.min_chain,
        t.max_chain,
        t.tatoeba_weight,
        t.held_back_per_mille,
        t.followers,
    );
    assert_eq!(values, (2, 3, 4, 20, 8));
    assert_eq!(
        (t.out.as_str(), t.held.as_str()),
        ("assets/words", "words/held")
    );
}

#[test]
fn every_language_gets_one_pack_the_full_ones_first() {
    let isos = Manifest::parse(MANIFEST).unwrap().isos();
    assert_eq!(
        isos.get(..3),
        Some(&["ar", "en", "fr"].map(String::from)[..])
    );
    assert_eq!(isos.len(), 42, "3 full and 39 word-only packs");
    assert_eq!(isos.iter().filter(|i| *i == "ar").count(), 1);
    assert!(isos.contains(&"fil".to_string()) && isos.contains(&"de".to_string()));
}

#[test]
fn a_bad_language_text_is_refused() {
    let text = fixture::manifest_text("");
    let head = "[lang.fr.text]\nchain_budget_kb = 1\n";
    let good = format!("{head}tatoeba = {{ file = \"d/fr.aff\", max_sentences = 1 }}\n");
    assert!(Manifest::parse(&format!("{text}{good}")).is_ok());
    let bad = [
        good.replace("chain_budget_kb = 1", "chain_budget_kb = 0"),
        good.replace("max_sentences = 1", "max_sentences = 0"),
        good.replace("d/fr.aff", "d/nope.txt"),
        head.to_string(),
        format!("{head}wiki = {{ files = [], max_bytes = 1 }}\n"),
        format!("{head}wiki = {{ files = [\"d/fr.aff\"], max_bytes = 0 }}\n"),
    ];
    for lang in bad {
        let result = Manifest::parse(&format!("{text}{lang}"));
        assert!(
            matches!(result, Err(WordsError::Manifest(_))),
            "{lang} was accepted"
        );
    }
}
