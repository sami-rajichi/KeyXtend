//! G20: a clipboard listener keeps every copy, skips marked ones, and an older copy pastes back.
#![cfg(windows)]

use std::ops::RangeInclusive;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};

use crate::apps::{AppKind, Ctx};
use crate::cliplisten::{Listener, MARKS, Reads, Tally};
use crate::featcfg::G20;
use crate::probe::{self, Doc};
use crate::simuser::keys_ours;
use crate::win::sleep_ms;
use crate::winclip::{self, line};
use crate::{clip, clipkeep, keys, readback};

/// The app G20 copies in and pastes into.
pub const APPS: [AppKind; 1] = [AppKind::Notepad];

/// Test text number `i`.
fn text(prefix: &str, i: usize) -> String {
    format!("{prefix}{i:02}")
}

/// The copies kx-gates writes itself, then the ones copied in Notepad.
fn halves(g: &G20) -> (RangeInclusive<usize>, RangeInclusive<usize>) {
    let own = g.copies / 2;
    (1..=own, own + 1..=g.copies)
}

/// The entry `n` places back from the newest (1 is the newest).
fn older(entries: &[String], n: usize) -> Option<&String> {
    let i = entries.len().checked_sub(n)?;
    (n > 0).then(|| entries.get(i)).flatten()
}

/// Where the kept entries first differ from `want`; the entry is shown only when it is a test text.
fn first_break(entries: &[String], want: &[String]) -> Option<Value> {
    let got = |i: usize| entries.get(i).map(|e| line(e));
    let at =
        (0..entries.len().max(want.len())).find(|&i| got(i) != want.get(i).map(String::as_str))?;
    let shown = match got(at) {
        None => json!("(missing)"),
        Some(g) if want.iter().any(|w| w == g) => json!(g),
        Some(_) => json!("(not a test text)"),
    };
    Some(json!({ "at": at, "got": shown }))
}

/// Waits for the listener's report of one change and counts it; notes a miss.
fn hear(ctx: &Ctx, l: &Listener, tally: &mut Tally, notes: &mut Vec<String>) {
    match l.next(ctx.cfg.g20.event_wait_ms) {
        Some(Ok(u)) => tally.add(u),
        Some(Err(e)) => notes.push(e),
        None => notes.push("the listener heard nothing".to_string()),
    }
}

/// Copies Notepad's lines one by one with Home, Shift+End and Ctrl+C.
fn copy_lines(
    ctx: &Ctx,
    doc: &Doc,
    l: &Listener,
    tally: &mut Tally,
    notes: &mut Vec<String>,
) -> Result<(), String> {
    let (k, ek, app) = (&ctx.cfg.keys, &ctx.cfg.edit_keys, &doc.app);
    probe::front(ctx, app)?;
    keys_ours(app)?;
    keys::combo(&ek.doc_start)?;
    for _ in halves(&ctx.cfg.g20).1 {
        keys_ours(app)?;
        keys::combo(&ek.line_start)?;
        keys::combo(&ek.select_line)?;
        sleep_ms(ctx.cfg.timing.key_settle_ms);
        keys_ours(app)?;
        keys::combo(&k.copy)?;
        hear(ctx, l, tally, notes);
        keys_ours(app)?;
        keys::combo(&ek.next_line)?;
    }
    Ok(())
}

/// Copies with no wait between them; returns how many changes the listener heard.
fn burst(ctx: &Ctx, l: &Listener, notes: &mut Vec<String>) -> Result<usize, String> {
    let (g, t) = (&ctx.cfg.g20, &ctx.cfg.timing);
    for i in 1..=g.burst {
        let quick = text(&g.prefix, g.copies + i);
        clip::write_text(Some(&quick), t.read_wait_ms, t.poll_ms)
            .map_err(|e| format!("burst {i}: {e}"))?;
    }
    let mut b = Tally::default();
    for _ in 0..g.burst {
        hear(ctx, l, &mut b, notes);
    }
    Ok(b.seen)
}

/// Puts the chosen older entry back on the clipboard, pastes it at the end, and checks the saved file.
fn paste_back(ctx: &Ctx, doc: &Doc, entry: &str) -> Result<bool, String> {
    let (t, ek) = (&ctx.cfg.timing, &ctx.cfg.edit_keys);
    clip::write_text(Some(entry), t.read_wait_ms, t.poll_ms)
        .map_err(|e| format!("paste back: {e}"))?;
    probe::front(ctx, &doc.app)?;
    keys_ours(&doc.app)?;
    keys::combo(&ek.doc_end)?;
    keys::combo(&ctx.cfg.keys.enter)?;
    keys::combo(&ek.paste)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    readback::saved_until(ctx, &doc.app, |text| text.trim_end().ends_with(line(entry)))
}

/// What the listener hears: kx-gates' copies, Notepad's copies, the marked copies, then a burst.
fn listen(ctx: &Ctx, doc: &Doc) -> Result<(Tally, usize, Vec<String>), String> {
    let (g, t) = (&ctx.cfg.g20, &ctx.cfg.timing);
    let reads = Reads {
        max_chars: g.max_chars,
        open_ms: t.read_wait_ms,
        poll_ms: t.poll_ms,
    };
    let l = Listener::start(reads)?;
    let (mut tally, mut notes) = (Tally::default(), Vec::new());
    for i in halves(g).0 {
        clip::write_text(Some(&text(&g.prefix, i)), t.read_wait_ms, t.poll_ms)
            .map_err(|e| format!("copy {i}: {e}"))?;
        hear(ctx, &l, &mut tally, &mut notes);
    }
    copy_lines(ctx, doc, &l, &mut tally, &mut notes)?;
    for i in 0..g.excluded {
        let (hidden, mark) = (text(&g.hidden_prefix, i + 1), [MARKS[i % MARKS.len()]]);
        clip::write_marked(Some(&hidden), &mark, t.read_wait_ms, t.poll_ms)
            .map_err(|e| format!("marked copy {}: {e}", i + 1))?;
        hear(ctx, &l, &mut tally, &mut notes);
    }
    let heard = burst(ctx, &l, &mut notes)?;
    Ok((tally, heard, notes))
}

/// The whole run on Notepad: listen, then paste an older copy back.
fn drive(ctx: &Ctx, doc: &Doc) -> Result<Value, String> {
    let g = &ctx.cfg.g20;
    let (tally, heard_burst, notes) = listen(ctx, doc)?;
    let want: Vec<String> = (1..=g.copies).map(|i| text(&g.prefix, i)).collect();
    let order_break = first_break(&tally.entries, &want);
    let in_order = order_break.is_none();
    let leaked = tally
        .entries
        .iter()
        .any(|e| e.starts_with(&g.hidden_prefix));
    let entry = older(&tally.entries, g.paste_back).ok_or("too few entries to paste one back")?;
    let pasted = paste_back(ctx, doc, entry)?;
    // Only a known test text is logged; anything else copied meanwhile is not ours to show.
    let known = want.iter().any(|w| w == line(entry));
    let shown = if known {
        json!(line(entry))
    } else {
        json!("(not a test text)")
    };
    let pass = tally.seen == g.copies + g.excluded
        && tally.skipped == g.excluded
        && in_order
        && !leaked
        && heard_burst == g.burst
        && pasted;
    Ok(json!({
        "gate": "G20", "app": doc.app.kind.name(), "pass": pass,
        "seen": tally.seen, "skipped": tally.skipped, "kept_in_order": in_order, "order_break": order_break,
        "hidden_kept": leaked, "burst_heard": heard_burst, "pasted": shown, "paste_ok": pasted,
        "notes": notes,
    }))
}

/// Removes this run's test texts from Windows clipboard history; reports how many, and any marked one Windows kept.
fn forget(ctx: &Ctx, since: i64) -> Value {
    let g = &ctx.cfg.g20;
    let hidden = AtomicUsize::new(0);
    let removed = probe::forget_copies(ctx, since, &|t| {
        if t.starts_with(&g.hidden_prefix) {
            hidden.fetch_add(1, Ordering::Relaxed);
        }
        winclip::has_prefix(t, &[&g.prefix, &g.hidden_prefix])
    });
    json!({ "removed": removed, "windows_kept_hidden": hidden.load(Ordering::Relaxed) })
}

/// Runs G20 on `name`, keeping the owner's clipboard; the history is cleaned even when the run fails.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    let (g, since) = (&ctx.cfg.g20, winclip::now());
    let lines: Vec<String> = halves(g).1.map(|i| text(&g.prefix, i)).collect();
    let doc = probe::open_text(ctx, kind, &lines.join("\n"))?;
    let result = probe::run_on(ctx, doc, |doc| clipkeep::keep(ctx.cfg, || drive(ctx, doc)));
    let history = forget(ctx, since);
    let (mut v, notes) = result.map_err(|e| format!("{e}; history: {history}"))?;
    v["history"] = history;
    v["clean_up"] = json!(notes);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries(n: usize) -> Vec<String> {
        (1..=n).map(|i| text("kx-clip-", i)).collect()
    }

    #[test]
    fn test_texts_are_numbered_with_two_digits() {
        assert_eq!(text("kx-clip-", 7), "kx-clip-07");
        assert_eq!(text("kx-clip-", 20), "kx-clip-20");
    }

    #[test]
    fn the_older_entry_counts_back_from_the_newest() {
        let e = entries(20);
        assert_eq!(older(&e, 1).map(String::as_str), Some("kx-clip-20"));
        assert_eq!(older(&e, 5).map(String::as_str), Some("kx-clip-16"));
        assert_eq!(older(&e, 21), None);
        assert_eq!(older(&e, 0), None);
    }

    #[test]
    fn first_break_finds_the_first_wrong_entry_and_hides_other_text() {
        let want = entries(3);
        let mut got = entries(3);
        assert_eq!(first_break(&got, &want), None);
        got[1] = "kx-clip-03".into();
        assert_eq!(
            first_break(&got, &want),
            Some(json!({ "at": 1, "got": "kx-clip-03" }))
        );
        got[1] = "private".into();
        assert_eq!(
            first_break(&got, &want),
            Some(json!({ "at": 1, "got": "(not a test text)" }))
        );
        assert_eq!(
            first_break(&got[..1], &want),
            Some(json!({ "at": 1, "got": "(missing)" }))
        );
    }
}
