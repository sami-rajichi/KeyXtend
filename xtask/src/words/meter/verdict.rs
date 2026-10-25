//! The verdict: each gate language asked for must save the target share with the seed alone; a gate language with no
//! held sentences fails, and the other languages are reported only.

use std::collections::BTreeMap;

use crate::words::manifest::MeterRules;

/// Why each gate language among `asked` failed, given each measured language's saved share in percent; empty when
/// the gate passes. The reasons name languages and numbers only.
pub(crate) fn failures(
    rules: &MeterRules,
    asked: &[String],
    saved: &BTreeMap<String, f64>,
) -> Vec<String> {
    let target = f64::from(rules.min_saved_pct);
    let gated = (rules.gate.iter()).filter(|iso| asked.contains(iso));
    gated
        .filter_map(|iso| match saved.get(iso) {
            None => Some(format!("{iso} has no held sentences to measure")),
            Some(&pct) if pct < target => Some(format!(
                "{iso} saved {pct:.1}% of the presses, under {target}%"
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
