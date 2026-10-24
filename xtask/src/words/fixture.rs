//! Tiny fixtures for the `words` tests, built in memory: wordfreq lists, dictionaries, rules and manifests.

use std::io::Write;

use flate2::Compression;
use flate2::write::GzEncoder;
use rmpv::Value;

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

/// The project's cleaning rules, with a coverage line over the top 3 words that prints 2.
pub(crate) fn rules() -> Rules {
    Rules {
        drop_chars: vec!['@', '/', '.'],
        apostrophe: '\'',
        steps_per_zipf: 25,
        coverage_top: 3,
        coverage_show: 2,
    }
}

/// The SHA-256 of `abc`, from FIPS 180-2's first example.
pub(crate) const ABC_SHA256: &str =
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

/// A manifest with these `[sources]` lines, a French language and a small list pattern.
pub(crate) fn manifest_with(sources: &str) -> Result<Manifest, super::WordsError> {
    Manifest::parse(&manifest_text(sources))
}

/// The text of [`manifest_with`].
pub(crate) fn manifest_text(sources: &str) -> String {
    format!(
        r#"
[rules]
drop_chars = ["@", "/", "."]
apostrophe = "'"
steps_per_zipf = 25
coverage_top = 3
coverage_show = 2

[fetch]
root_env = "KX_TEST_NO_SUCH_VAR"
curl = "curl.exe"
fallback = "kx-test-no-such-curl"
args = ["--output"]
part = "part"

[small]
list = "w/small_{{iso}}.gz"
max_words = 2

[lang.fr]
list = "w/large_fr.gz"
aff = "d/fr.aff"
dic = "d/fr.dic"
max_words = 5
elisions = ["l", "qu"]

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
