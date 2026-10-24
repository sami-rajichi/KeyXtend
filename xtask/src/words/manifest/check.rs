//! The manifest's value checks: each threshold in its range, a safe fetch setup, https bases and sound source paths.

use super::super::WordsError;
use super::super::wordfreq::ZIPF_TOP_CB;
use super::{FetchConfig, HASH_HEX, Rules, bad};
use crate::devtools::paths::{is_plain_name, is_relative_plain};

/// The only scheme a base may use, so a download can never fall back to plain HTTP.
const HTTPS: &str = "https://";
/// A main script needs more than this share of the words, so only one script can be it.
const MIN_SHARE: f64 = 0.5;

/// Refuses a rule threshold that is zero, not finite or out of its range.
pub(super) fn rules(rules: &Rules) -> Result<(), WordsError> {
    let share = rules.main_script_share;
    let checks = [
        ("max_gap_cb", (1..=ZIPF_TOP_CB).contains(&rules.max_gap_cb)),
        ("main_script_share", share > MIN_SHARE && share <= 1.0),
        ("min_case_seen", rules.min_case_seen > 0),
        (
            "piece_ratio",
            rules.piece_ratio.is_finite() && rules.piece_ratio > 1.0,
        ),
        (
            "piece_min_expected",
            rules.piece_min_expected.is_finite() && rules.piece_min_expected > 0.0,
        ),
        ("coverage_top", rules.coverage_top > 0),
        ("show", rules.show > 0),
    ];
    match checks.iter().find(|(_, ok)| !ok) {
        Some((name, _)) => Err(bad(&format!("[rules] {name} is out of its range"))),
        None => Ok(()),
    }
}

/// Refuses an empty program or flag, and a part extension that is not one plain name.
pub(super) fn fetch(fetch: &FetchConfig) -> Result<(), WordsError> {
    let names = [
        &fetch.root_env,
        &fetch.curl,
        &fetch.fallback,
        &fetch.size_flag,
        &fetch.out_flag,
        &fetch.url_flag,
    ];
    if names.iter().any(|n| n.is_empty()) || !is_plain_name(&fetch.part) {
        return Err(bad(
            "[fetch] needs every program and flag, and a plain part name",
        ));
    }
    Ok(())
}

/// Refuses a base that is not an https URL.
pub(super) fn bases<'a>(mut bases: impl Iterator<Item = &'a String>) -> Result<(), WordsError> {
    match bases.find(|b| !b.starts_with(HTTPS)) {
        Some(base) => Err(bad(&format!("{base}: a base must start with {HTTPS}"))),
        None => Ok(()),
    }
}

/// Refuses a source path that is not relative and plain, or a hash that is not lowercase hex.
pub(super) fn source_path(rel: &str, sha256: &str) -> Result<(), WordsError> {
    let hex = |c: char| c.is_ascii_digit() || ('a'..='f').contains(&c);
    if !is_relative_plain(rel) {
        return Err(bad(&format!(
            "{rel:?} is not a relative path of plain names"
        )));
    }
    if sha256.len() != HASH_HEX || !sha256.chars().all(hex) {
        return Err(bad(&format!(
            "{rel}: the SHA-256 is not {HASH_HEX} lowercase hex digits"
        )));
    }
    Ok(())
}
