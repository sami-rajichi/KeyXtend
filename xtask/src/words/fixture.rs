//! Tiny fixtures for the `words` tests, built in memory: wordfreq lists, dictionaries, rules and manifests.

pub(crate) mod text;

use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use rmpv::Value;

use kx_mod_predict::fold;

use super::manifest::{Manifest, Rules};

/// The header wordfreq writes.
pub(crate) fn header() -> Value {
    Value::Map(vec![
        (Value::from("format"), Value::from("cB")),
        (Value::from("version"), Value::from(1)),
    ])
}

/// A gzipped msgpack list: `header`, then one list of words per bucket.
pub(crate) fn list_with(header: Value, buckets: &[&[&str]]) -> Vec<u8> {
    let lists = buckets
        .iter()
        .map(|words| Value::Array(words.iter().map(|w| Value::from(*w)).collect()));
    let all = Value::Array(std::iter::once(header).chain(lists).collect());
    let mut raw = Vec::new();
    rmpv::encode::write_value(&mut raw, &all).unwrap();
    let mut gz = GzEncoder::new(Vec::new(), Compression::fast());
    gz.write_all(&raw).unwrap();
    gz.finish().unwrap()
}

/// A wordfreq list with the right header.
pub(crate) fn list(buckets: &[&[&str]]) -> Vec<u8> {
    list_with(header(), buckets)
}

/// The smallest affix file: UTF-8 and no rules.
pub(crate) const AFF: &str = "SET UTF-8\n";

/// A word file holding `words`.
pub(crate) fn dic(words: &[&str]) -> String {
    let mut text = format!("{}\n", words.len());
    for word in words {
        text.push_str(word);
        text.push('\n');
    }
    text
}

/// The project's rules, with a coverage line over the top 3 words and report lines that print 2.
pub(crate) fn rules() -> Rules {
    Rules {
        drop_chars: vec!['@', '/', '.'],
        apostrophe: fold::apostrophe().unwrap(),
        max_gap_cb: 100,
        main_script_share: 0.9,
        min_case_seen: 20,
        twin_ratio: 10.0,
        piece_ratio: 20.0,
        piece_min_seen: 100,
        coverage_top: 3,
        show: 2,
    }
}

/// The SHA-256 of `abc`, from FIPS 180-2's first example.
pub(crate) const ABC_SHA256: &str =
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

/// A manifest with these `[sources]` lines, a French language and a small list pattern.
pub(crate) fn manifest_with(sources: &str) -> Result<Manifest, super::WordsError> {
    Manifest::parse(&manifest_text(sources))
}

/// The `[text]`, `[packs]`, `[meter]` and `[credits]` tables of the fixture manifest; the credits read the fixture's
/// French files.
pub(crate) const TEXT: &str = r#"[text]
sentence_ends = [".", "?"]
min_chain = 2
max_chain = 3
tatoeba_weight = 4
held_back_per_mille = 20
max_live_chains = 100
max_seen_sentences = 1000
skew_ratio = 10.0
report_contexts = 2
report_followers = 2
followers = 2
null = "-"
held = "held"
out = "out"

[packs]
download_kb = 1000
level = 6

[meter]
gate = ["en"]
min_saved_pct = 40
max_sentences = 2
percentiles = [50, 99]
ceiling_us = 10000
speed_learns = 10
speed_samples = 2

[credits]
file = "CREDITS.txt"
authors = "AUTHORS.txt"
licence = "LICENSE.txt"
licence_text = ["Shared alike.", "See the credits."]
wordfreq = { file = "d/fr.aff", from = "A", to = "B" }
scowl = { file = "d/fr.dic", from = "A", to = "B" }
citations = { file = "d/fr.aff", from = "C" }
"#;

/// The fixture manifest's `[fetch]` table.
const FETCH: &str = r#"[fetch]
root_env = "KX_TEST_NO_SUCH_VAR"
curl = "curl.exe"
fallback = "kx-test-no-such-curl"
args = ["-q", "--fail"]
size_flag = "--max-filesize"
out_flag = "--output"
url_flag = "--url"
part = "part"
"#;

/// The text of [`manifest_with`].
pub(crate) fn manifest_text(sources: &str) -> String {
    format!(
        r#"
[rules]
drop_chars = ["@", "/", "."]
max_gap_cb = 100
main_script_share = 0.9
min_case_seen = 20
twin_ratio = 10.0
piece_ratio = 20.0
piece_min_seen = 100
coverage_top = 3
show = 2

{TEXT}
{FETCH}
[small]
list = "w/small_{{iso}}.gz"
max_words = 2
words_budget_kb = 100

[lang.fr]
list = "w/large_fr.gz"
aff = "d/fr.aff"
dic = "d/fr.dic"
max_words = 5
words_budget_kb = 200

[bases]
test = "https://example.invalid/data/"

[sources]
"w/large_fr.gz" = {{ base = "test", sha256 = "{ABC_SHA256}", size = 3 }}
"d/fr.aff" = {{ base = "test", sha256 = "{ABC_SHA256}", size = 3 }}
"d/fr.dic" = {{ base = "test", sha256 = "{ABC_SHA256}", size = 3 }}
{sources}
"#
    )
}

/// A source `d/abc.txt` holding `abc`, downloaded from the test base.
pub(crate) fn abc_source() -> String {
    format!(r#""d/abc.txt" = {{ base = "test", dir = "x/", sha256 = "{ABC_SHA256}", size = 3 }}"#)
}

/// An affix file with one prefix, `ال`, under flag `A`.
pub(crate) const AFF_AL: &str = "SET UTF-8\nPFX A Y 1\nPFX A 0 ال .\n";

/// An affix file with one suffix, `انا`, under flag `B`: Arabic suffixes are often words too.
pub(crate) const AFF_ANA: &str = "SET UTF-8\nSFX B Y 1\nSFX B 0 انا .\n";
