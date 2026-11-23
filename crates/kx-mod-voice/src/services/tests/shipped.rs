//! The shipped rows as other tests read them (final review I2), so a check-day refresh of `services.toml` edits no
//! test; `research.rs` alone pins the dated facts.

use crate::cloud::pick_text;
use crate::services::{Model, Service, table};

/// The day the tests' entries were switched on, before any row's facts day.
pub(crate) const ON_DAY: &str = "2026-11-01";

/// The first row the Cloud page offers (edge 24).
pub(crate) fn first() -> &'static Service {
    table().unwrap().offered().next().unwrap()
}

/// `row`'s listed model `n`, in the row's order.
pub(crate) fn model(row: &Service, n: usize) -> &Model {
    &row.models[n]
}

/// The pick that names `row`'s listed model `n`.
pub(crate) fn pick(row: &Service, n: usize) -> String {
    pick_text(&row.id, &model(row, n).id)
}

/// `row`'s `[voice] services` entry, on at its own terms number, with `extra` fields after.
pub(crate) fn entry(row: &Service, extra: &str) -> String {
    let (id, terms) = (&row.id, row.terms);
    format!(r#""{id}" = {{ on = {ON_DAY}, terms = {terms}{extra} }}"#)
}

/// The id of shipped row `id`'s own card warning, such as `ElevenLabs`' profile one; empty for none.
pub(crate) fn warn_id(id: &str) -> &'static str {
    let warn = table().unwrap().get(id).and_then(|r| r.facts.warn.as_ref());
    warn.map_or("", |w| w.id.as_str())
}
