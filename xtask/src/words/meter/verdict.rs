//! The verdict: each gate language judged must save the target share with the seed alone. A full run judges every
//! gate language, a `--lang` run only the one asked for; one not measured fails, and the others are reported only.

use std::collections::BTreeMap;

use super::tally::shown_pct;
use crate::words::manifest::MeterRules;

/// Why each gate language judged failed, given each measured language's saved share in percent: every gate language
/// in a full run, or only `lang` when one was asked for. The reasons name languages and numbers only.
pub(crate) fn failures(
    rules: &MeterRules,
    lang: Option<&str>,
    saved: &BTreeMap<String, f64>,
) -> Vec<String> {
    let target = rules.min_saved_pct;
    let judged = (rules.gate.iter()).filter(|iso| lang.is_none_or(|l| l == iso.as_str()));
    judged
        .filter_map(|iso| match saved.get(iso) {
            None => Some(format!(
                "{iso} was not measured: no pack in the index or no held sentences"
            )),
            Some(&pct) if pct < f64::from(target) => Some(format!(
                "{iso} saved {}% of the presses, under {target}%",
                shown_pct(pct)
            )),
            Some(_) => None,
        })
        .collect()
}

/// The line that says the gate passed.
pub(crate) fn passed(rules: &MeterRules) -> String {
    let gate = rules.gate.join(", ");
    format!(
        "gate passed: {gate} each saved at least {}% where asked",
        rules.min_saved_pct
    )
}
