//! G22 typing check: the test sentences typed into Notepad in one batch, then one character at a time; focus is checked before each.
//! Only counts are reported; the sentences are our own test text.

use std::collections::BTreeMap;
use std::time::Instant;

use serde_json::{Value, json};
use spike_core::inject;
use spike_core::uia::Uia;

use crate::apps::{AppKind, Ctx, Opened};
use crate::g22::{added, ms_since, words_in};
use crate::simuser::keys_ours;
use crate::{probe, readback};

/// Name that runs this check instead of a face.
pub const TYPING: &str = "typing";

/// What one focus check covers: each character when paced, else the whole text in one batch.
fn parts(text: &str, gap_ms: u64) -> Vec<String> {
    if gap_ms == 0 {
        vec![text.to_string()]
    } else {
        text.chars().map(String::from).collect()
    }
}

/// Types `text` with `gap_ms` between characters, checking before each part that Notepad still holds the keyboard.
fn type_ours(app: &Opened, text: &str, gap_ms: u64) -> Result<(), String> {
    for p in parts(text, gap_ms) {
        keys_ours(app)?;
        inject::text_paced(&p, gap_ms)?;
    }
    Ok(())
}

/// Types `text` with `gap_ms` between characters and counts what Notepad shows once it settles.
fn try_one(ctx: &Ctx, uia: &Uia, app: &Opened, text: &str, gap_ms: u64) -> Result<Value, String> {
    let before = uia
        .doc_text(app.hwnd, ctx.cfg.g22.max_chars)
        .ok_or("Notepad's text cannot be read")?;
    let typed = format!("{text}{}", ctx.spike.voice.type_suffix);
    let t0 = Instant::now();
    type_ours(app, &typed, gap_ms)?;
    let typed_ms = ms_since(t0);
    let now = words_in(ctx, uia, app, &before).map_or(before.clone(), |(_, t)| t);
    let got = added(&before, &now);
    Ok(json!({
        "gap_ms": gap_ms, "typed_ms": typed_ms, "want_chars": text.chars().count(),
        "got_chars": got.chars().count(), "exact": got == text.trim(),
    }))
}

/// How many tries typed exactly, per gap.
fn exact_by_gap(rows: &[Value]) -> BTreeMap<u64, usize> {
    let mut by = BTreeMap::new();
    for r in rows {
        if let Some(gap) = r["gap_ms"].as_u64() {
            *by.entry(gap).or_default() += usize::from(r["exact"] == true);
        }
    }
    by
}

/// Every gap over every sentence, then Notepad's text length through UIA and from its saved file.
fn drive(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let uia = Uia::new()?;
    probe::front(ctx, app)?;
    let g = &ctx.cfg.g22;
    let mut rows = Vec::new();
    for &gap in &g.type_gaps_ms {
        for s in &g.sentences {
            let r = try_one(ctx, &uia, app, &s.text, gap);
            rows.push(r.unwrap_or_else(|e| json!({ "gap_ms": gap, "error": e })));
        }
    }
    let uia_chars = uia
        .doc_text(app.hwnd, g.max_chars)
        .map(|t| t.chars().count());
    let file = keys_ours(app).and_then(|()| readback::read_back(ctx, app));
    let by = exact_by_gap(&rows);
    let pass = by.values().any(|&n| n == g.sentences.len());
    Ok(json!({
        "pass": pass, "exact_by_gap": by, "tries": rows, "uia_chars": uia_chars,
        "file_chars": file.map(|t| t.chars().count()),
    }))
}

/// Runs the typing check in a fresh Notepad file.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let doc = probe::open_text(ctx, AppKind::Notepad, "")?;
    let (mut v, notes) = probe::run_on(ctx, doc, |doc| drive(ctx, &doc.app))?;
    v["gate"] = json!("G22");
    v["face"] = json!(TYPING);
    v["clean_up"] = json!(notes);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_tries_are_counted_per_gap() {
        let rows = [
            json!({ "gap_ms": 0, "exact": false }),
            json!({ "gap_ms": 0, "exact": true }),
            json!({ "gap_ms": 10, "exact": true }),
            json!({ "gap_ms": 10, "error": "x" }),
        ];
        let by = exact_by_gap(&rows);
        assert_eq!((by[&0], by[&10]), (1, 1));
    }

    #[test]
    fn a_gap_checks_focus_before_each_character_and_no_gap_once() {
        let g = crate::config::load().expect("harness.toml loads").g22;
        for s in &g.sentences {
            assert_eq!(parts(&s.text, 0), [s.text.as_str()], "one batch");
            let each = parts(&s.text, 1);
            assert_eq!(each.len(), s.text.chars().count());
            assert_eq!(each.concat(), s.text, "no character is split or lost");
        }
    }
}
