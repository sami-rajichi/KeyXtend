//! The manifest's value checks: each threshold in its range, a safe fetch setup, https bases, sound source paths,
//! text limits and credits.

use std::collections::BTreeMap;

use flate2::Compression;
use kx_mod_predict::pack::layout::{MAX_CHAIN, MIN_CHAIN};
use kx_module_api::ports::{PackIndex, fits_iso};

use super::super::WordsError;
use super::super::wordfreq::ZIPF_TOP_CB;
use super::text::{PER_MILLE, PERCENT};
use super::{
    Credits, FetchConfig, HASH_HEX, LangText, MeterRules, Packs, Rules, Section, Source, TextRules,
    bad,
};
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
            "twin_ratio",
            rules.twin_ratio.is_finite() && rules.twin_ratio >= 1.0,
        ),
        (
            "piece_ratio",
            rules.piece_ratio.is_finite() && rules.piece_ratio > 1.0,
        ),
        ("piece_min_seen", rules.piece_min_seen > 0),
        ("coverage_top", rules.coverage_top > 0),
        ("show", rules.show > 0),
    ];
    match checks.iter().find(|(_, ok)| !ok) {
        Some((name, _)) => Err(bad(&format!("[rules] {name} is out of its range"))),
        None => Ok(()),
    }
}

/// Refuses text rules out of their ranges: chains the pack cannot hold, a zero weight, cap or follower count, a
/// held-back share of a thousand or more, and folders that are not plain relative paths.
pub(super) fn text(text: &TextRules) -> Result<(), WordsError> {
    let chains = MIN_CHAIN <= text.min_chain && text.min_chain <= text.max_chain;
    let checks = [
        ("sentence_ends", !text.sentence_ends.is_empty()),
        ("min_chain", chains && text.max_chain <= MAX_CHAIN),
        ("tatoeba_weight", text.tatoeba_weight > 0),
        ("held_back_per_mille", text.held_back_per_mille < PER_MILLE),
        ("max_live_chains", text.max_live_chains > 0),
        ("max_seen_sentences", text.max_seen_sentences > 0),
        (
            "skew_ratio",
            text.skew_ratio.is_finite() && text.skew_ratio >= 1.0,
        ),
        ("report_contexts", text.report_contexts > 0),
        ("report_followers", text.report_followers > 0),
        ("followers", text.followers > 0),
        ("null", !text.null.is_empty()),
        ("held", is_relative_plain(&text.held)),
        ("out", is_relative_plain(&text.out)),
    ];
    match checks.iter().find(|(_, ok)| !ok) {
        Some((name, _)) => Err(bad(&format!("[text] {name} is out of its range"))),
        None => Ok(()),
    }
}

/// Refuses a gate code no pack could carry, a share past a whole, a zero cap, and percentiles that are not rising
/// within a whole.
pub(super) fn meter(meter: &MeterRules) -> Result<(), WordsError> {
    let rising = (meter.percentiles.windows(2)).all(|w| matches!(w, [a, b] if a < b));
    let within = |p: &u32| (1..=PERCENT).contains(p);
    let checks = [
        ("gate", meter.gate.iter().all(|iso| fits_iso(iso))),
        ("min_saved_pct", meter.min_saved_pct <= PERCENT),
        ("max_sentences", meter.max_sentences > 0),
        (
            "percentiles",
            !meter.percentiles.is_empty() && rising && meter.percentiles.iter().all(within),
        ),
        ("ceiling_us", meter.ceiling_us > 0),
        ("speed_learns", meter.speed_learns > 0),
        ("speed_samples", meter.speed_samples > 0),
    ];
    match checks.iter().find(|(_, ok)| !ok) {
        Some((name, _)) => Err(bad(&format!("[meter] {name} is out of its range"))),
        None => Ok(()),
    }
}

/// Refuses a zero download budget and a deflate level past the best one.
pub(super) fn packs(packs: &Packs) -> Result<(), WordsError> {
    let best = Compression::best().level();
    if packs.download_kb > 0 && packs.level <= best {
        Ok(())
    } else {
        Err(bad(&format!(
            "[packs] needs a download budget and a level of 0 to {best}"
        )))
    }
}

/// Refuses an alias of a pack not built, and aliases the pack index refuses: a bad code, or a pack's own code.
pub(super) fn aliases(packs: &Packs, isos: &[String]) -> Result<(), WordsError> {
    let built = packs.aliases.keys().all(|pack| isos.contains(pack));
    let isos: Vec<&str> = isos.iter().map(String::as_str).collect();
    if built && PackIndex::new(&isos, &packs.aliases).is_ok() {
        Ok(())
    } else {
        Err(bad(
            "[packs] aliases must name built packs and serve other codes",
        ))
    }
}

/// True when a language's text names a source and every limit is above zero.
pub(super) fn lang_text(text: &LangText) -> bool {
    let tatoeba = text.tatoeba.as_ref().is_none_or(|t| t.max_sentences > 0);
    let wiki = text
        .wiki
        .as_ref()
        .is_none_or(|w| !w.files.is_empty() && w.max_bytes > 0);
    let some = text.tatoeba.is_some() || text.wiki.is_some();
    text.chain_budget_kb > 0 && tatoeba && wiki && some
}

/// Refuses credit file names that are not plain, an empty licence, and sections of unknown files or empty marks.
pub(super) fn credits(
    credits: &Credits,
    sources: &BTreeMap<String, Source>,
) -> Result<(), WordsError> {
    let names = [&credits.file, &credits.authors, &credits.licence];
    let sections = [&credits.wordfreq, &credits.scowl, &credits.citations];
    let marked = |s: &Section| !s.from.is_empty() && s.to.as_ref().is_none_or(|t| !t.is_empty());
    let sound = sections
        .iter()
        .all(|s| sources.contains_key(&s.file) && marked(s));
    if names.iter().all(|n| is_plain_name(n)) && !credits.licence_text.is_empty() && sound {
        Ok(())
    } else {
        Err(bad(
            "[credits] needs plain file names, a licence text and sections of known sources",
        ))
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
