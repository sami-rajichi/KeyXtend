//! The manifest: the sources, languages and values a manifest refuses, and the credits template.

use flate2::Compression;
use kx_mod_predict::pack::layout::{MAX_CHAIN, MIN_CHAIN};

use super::super::fixture::{self, ABC_SHA256, abc_source};
use super::super::manifest::{Manifest, NO_LANGS};
use super::super::wordfreq::ZIPF_TOP_CB;
use super::super::{WordsError, credits};

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
fn a_small_list_of_a_full_language_is_refused_as_never_read() {
    let line =
        format!(r#""w/small_fr.gz" = {{ base = "test", sha256 = "{ABC_SHA256}", size = 3 }}"#);
    let result = fixture::manifest_with(&line);
    assert!(matches!(result, Err(WordsError::Manifest(_))), "{result:?}");
}

#[test]
fn a_full_language_whose_own_list_is_its_small_list_is_kept() {
    let text = fixture::manifest_text("").replace("w/large_fr.gz", "w/small_fr.gz");
    let manifest = Manifest::parse(&text);
    assert!(manifest.is_ok(), "{manifest:?}");
}

#[test]
fn a_manifest_of_no_language_says_so_rather_than_blaming_its_aliases() {
    let aliases = r#"aliases = { fr = ["wa"] }"#;
    let text = fixture::manifest_text("");
    assert!(text.contains(aliases));
    let text = text.replace(aliases, "aliases = {}");
    let (head, rest) = text.split_once("[lang.fr]").unwrap();
    let (_, tail) = rest.split_once("[bases]").unwrap();
    let result = Manifest::parse(&format!("{head}[lang]\n\n[bases]{tail}"));
    assert!(
        matches!(&result, Err(WordsError::Manifest(t)) if t == NO_LANGS),
        "{result:?}"
    );
}

/// Swaps that each put one rule, text or credits value out of its range.
const OUT_OF_RANGE: &[(&str, &str)] = &[
    ("max_gap_cb = 100", "max_gap_cb = 0"),
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
    (r#"from = "C" }"#, r#"from = "" }"#),
    ("sentence_ends = [\".\", \"?\"]", "sentence_ends = []"),
    ("max_seen_sentences = 1000", "max_seen_sentences = 0"),
    ("skew_ratio = 10.0", "skew_ratio = 0.9"),
    ("skew_ratio = 10.0", "skew_ratio = inf"),
    ("report_contexts = 2", "report_contexts = 0"),
    ("report_followers = 2", "report_followers = 0"),
    ("download_kb = 1000", "download_kb = 0"),
    ("words_budget_kb = 100", "words_budget_kb = 0"),
    ("words_budget_kb = 200", "words_budget_kb = 0"),
];

/// Swaps onto each edge the code owns, the Zipf top, the pack's chain lengths and the best deflate level, and one past
/// it: each pair's first is taken, its second refused.
fn code_edges() -> [(&'static str, String, String); 4] {
    let swap =
        |key: &str, edge: u32, past: u32| (format!("{key} = {edge}"), format!("{key} = {past}"));
    let (top, best) = (ZIPF_TOP_CB, Compression::best().level());
    let (min, max) = (u32::from(MIN_CHAIN), u32::from(MAX_CHAIN));
    let edges = [
        ("max_gap_cb = 100", swap("max_gap_cb", top, top + 1)),
        ("min_chain = 2", swap("min_chain", min, min - 1)),
        ("max_chain = 3", swap("max_chain", max, max + 1)),
        ("level = 6", swap("level", best, best + 1)),
    ];
    edges.map(|(from, (edge, past))| (from, edge, past))
}

#[test]
fn a_rule_threshold_out_of_range_is_refused() {
    let text = fixture::manifest_text("");
    let past = code_edges().map(|(from, _, past)| (from, past));
    let swaps = OUT_OF_RANGE.iter().map(|&(f, t)| (f, t.to_owned()));
    for (from, to) in swaps.chain(past) {
        assert!(text.contains(from), "{from}");
        let result = Manifest::parse(&text.replacen(from, &to, 1));
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
        ("main_script_share = 0.9", "main_script_share = 1.0"),
        ("min_case_seen = 20", "min_case_seen = 1"),
        ("twin_ratio = 10.0", "twin_ratio = 1.0"),
        ("max_chain = 3", "max_chain = 2"),
        ("held_back_per_mille = 20", "held_back_per_mille = 999"),
        ("skew_ratio = 10.0", "skew_ratio = 1.0"),
        ("level = 6", "level = 0"),
    ];
    let edges = code_edges().map(|(from, edge, _)| (from, edge));
    for (from, to) in swaps
        .map(|(f, t)| (f, t.to_owned()))
        .into_iter()
        .chain(edges)
    {
        let result = Manifest::parse(&text.replacen(from, &to, 1));
        assert!(result.is_ok(), "{to} was refused: {result:?}");
    }
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
