//! Tiny text sources built in memory: Tatoeba and Wikipedia files compressed with bzip2, and candidates by hand.

use std::fmt::Write as _;
use std::io::Write as _;

use bzip2::Compression;
use bzip2::write::BzEncoder;
use kx_module_api::text::Casing;

use super::super::cands::{Cand, Fate};

/// `bytes` compressed with bzip2.
pub(crate) fn bz2(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = BzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

/// A Tatoeba file of `(id, text, username)` entries, in its six-field layout.
pub(crate) fn tatoeba(entries: &[(&str, &str, &str)]) -> Vec<u8> {
    let mut text = String::new();
    for (id, sentence, user) in entries {
        let added = "2020-01-01 00:00:00";
        writeln!(text, "{id}\txx\t{sentence}\t{user}\t{added}\t{added}").unwrap();
    }
    bz2(text.as_bytes())
}

/// A Wikipedia dump of documents, each with its text or without the field, each after its action line.
pub(crate) fn wiki(docs: &[Option<&str>]) -> Vec<u8> {
    let mut text = String::new();
    for (id, doc) in docs.iter().enumerate() {
        let action = serde_json::json!({ "index": { "_id": id } });
        let body = doc.map_or_else(
            || serde_json::json!({ "title": "x", "source_text": "x" }),
            |t| serde_json::json!({ "title": "x", "source_text": "[[x]]", "text": t }),
        );
        writeln!(text, "{action}\n{body}").unwrap();
    }
    bz2(text.as_bytes())
}

/// A kept candidate with `forms`, its own first, its word in lower case by the root casing.
pub(crate) fn cand(forms: &[&str], bucket: u32, rank: usize) -> Cand {
    Cand {
        word: Casing::root().lower(forms.first().unwrap()),
        forms: forms.iter().map(|f| (*f).to_string()).collect(),
        bucket,
        rank,
        fate: Fate::Kept,
    }
}
