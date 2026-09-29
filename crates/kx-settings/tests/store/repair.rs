//! Repair bounds: unknown keys are set aside first, and a notice names at most `MAX_NAMED_KEYS` keys.

use crate::spec::{DEFAULTS, HOLD, defaults_with, open, reset, reset_more, table, with_file};
use kx_settings::{KEYS_SEPARATOR, MAX_NAMED_KEYS};

/// Unknown keys in the long tests: five more than one notice names.
const TYPOS: usize = MAX_NAMED_KEYS + 5;

/// The name of unknown key `n`, padded so the names sort in number order.
fn typo(n: usize) -> String {
    format!("typo{n:02}")
}

/// Lines setting the unknown keys `0..count` to 1.
fn typos(count: usize) -> String {
    let lines: Vec<String> = (0..count).map(|n| format!("{} = 1\n", typo(n))).collect();
    lines.concat()
}

/// The unknown keys `0..count`, joined as a notice joins them.
fn named(count: usize) -> String {
    let names: Vec<String> = (0..count).map(typo).collect();
    names.join(KEYS_SEPARATOR)
}

#[test]
fn a_typo_beside_a_pair_valid_only_together_rejects_only_the_typo() {
    let text = "[hold]\nmin_ms = 6000\nmax_ms = 9000\nmni_ms = 5\n";
    let (_dir, files) = with_file("typo", text);
    let (store, report) = open(&files);
    let pair = defaults_with("min_ms = 6000\nmax_ms = 9000");
    assert_eq!(store.get(HOLD), Some(&pair));
    assert_eq!(report.notices, vec![reset("mni_ms")]);
}

#[test]
fn a_typo_and_a_bad_value_beside_a_pair_keep_the_pair() {
    // The typo is set aside first, so leaving out the one bad value keeps the pair.
    let text = "[hold]\nextra = 1\nmin_ms = 6000\nmax_ms = 9000\nsound = \"loud\"\n";
    let (_dir, files) = with_file("typobad", text);
    let (store, report) = open(&files);
    let pair = defaults_with("min_ms = 6000\nmax_ms = 9000");
    assert_eq!(store.get(HOLD), Some(&pair));
    assert_eq!(report.notices, vec![reset("extra, sound")]);
}

#[test]
fn a_notice_names_at_most_the_limit_and_counts_the_rest() {
    let text = format!("[hold]\nmax_ms = 900\n{}", typos(TYPOS));
    let (_dir, files) = with_file("many", &text);
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&defaults_with("max_ms = 900")));
    let notice = reset_more(&named(MAX_NAMED_KEYS), TYPOS - MAX_NAMED_KEYS);
    assert_eq!(report.notices, vec![notice]);
}

#[test]
fn a_notice_at_the_limit_names_every_key_and_counts_none() {
    let text = format!("[hold]\n{}", typos(MAX_NAMED_KEYS));
    let (_dir, files) = with_file("limit", &text);
    let (_, report) = open(&files);
    assert_eq!(report.notices, vec![reset(&named(MAX_NAMED_KEYS))]);
}

#[test]
fn a_bad_version_names_at_most_the_limit_and_counts_the_rest() {
    let text = format!("[hold]\nversion = \"3\"\n{}", typos(TYPOS));
    let (_dir, files) = with_file("manyversion", &text);
    let (store, report) = open(&files);
    assert_eq!(store.get(HOLD), Some(&table(DEFAULTS)));
    let notice = reset_more(&named(MAX_NAMED_KEYS), TYPOS - MAX_NAMED_KEYS);
    assert_eq!(report.notices, vec![notice]);
}
