//! G22: the Mic button records in the worker, the caption bar shows on top without focus, and the words land in Notepad.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use spike_core::config::ToolButton;
use spike_core::langkey;
use spike_core::uia::Uia;
use spike_core::voicecfg::VoiceConfig;
use spike_core::window::foreground;

use crate::apps::{AppKind, Ctx, Opened};
use crate::facetools::{self, FaceWin};
use crate::featcfg::{self, G22, Sentence};
use crate::simuser::keys_ours;
use crate::win::{self, sleep_ms};
use crate::{g22bench, g22type, g25, probe, readback, wer};

/// Name that runs the bench instead of a face.
pub const BENCH: &str = "bench";
/// Milliseconds in a second.
const MS_PER_S: u64 = 1000;

/// Refuses waits that would let a recording stop by itself or give up before an engine does.
fn fits(g: &G22, v: &VoiceConfig) -> Result<(), String> {
    if g.record_ms.saturating_add(g.stop_wait_ms) >= v.record_max_s.saturating_mul(MS_PER_S) {
        return Err("g22.record_ms plus stop_wait_ms must stay below voice.record_max_s".into());
    }
    if g.text_wait_ms <= v.local.timeout_ms.max(v.cloud.timeout_ms) {
        return Err("g22.text_wait_ms must pass both engine timeouts".into());
    }
    Ok(())
}

/// The words `now` adds after `before`.
pub(crate) fn added(before: &str, now: &str) -> String {
    now.strip_prefix(before.trim_end())
        .unwrap_or(now)
        .trim()
        .to_string()
}

/// Milliseconds since `t`.
pub(crate) fn ms_since(t: Instant) -> u64 {
    u64::try_from(t.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Clicks Mic again until the click lands or `stop_wait_ms` runs out; the last error when it never landed.
fn stop_mic(ctx: &Ctx, fw: &FaceWin) -> Result<(), String> {
    let mut last = String::new();
    let press = || match fw.press(ctx, ToolButton::Mic) {
        Ok(()) => Some(()),
        Err(e) => {
            last = e;
            None
        }
    };
    let (wait, poll) = (ctx.cfg.g22.stop_wait_ms, ctx.cfg.timing.poll_ms);
    win::poll_until(wait, poll, press).ok_or(last)
}

/// Waits for the caption bar; true when it is on top over its whole box, else what covers it.
fn bar_seen(ctx: &Ctx, wait_ms: u64) -> (bool, Option<String>) {
    let title = &ctx.spike.voice.caption.title;
    let poll = ctx.cfg.timing.poll_ms;
    let Some(bar) = win::poll_until(wait_ms, poll, || facetools::shown(title)) else {
        return (false, Some("the caption bar did not show".into()));
    };
    match win::over(bar) {
        Some(w) if w == bar => (true, None),
        other => (false, other.map(win::describe)),
    }
}

/// Waits until the caption bar hides, so the next sentence sees its own bar.
fn bar_gone(ctx: &Ctx) -> bool {
    let c = &ctx.spike.voice.caption;
    let wait = c.hide_ms.saturating_add(ctx.cfg.g22.pause_ms);
    let hidden = || facetools::shown(&c.title).is_none().then_some(());
    win::poll_until(wait, ctx.cfg.timing.poll_ms, hidden).is_some()
}

/// Waits for Notepad's text to change, then to settle; the ms to the first change and the final text.
pub(crate) fn words_in(ctx: &Ctx, uia: &Uia, app: &Opened, before: &str) -> Option<(u64, String)> {
    let (g, t0) = (&ctx.cfg.g22, Instant::now());
    let read = || uia.doc_text(app.hwnd, g.max_chars);
    let changed = || read().filter(|t| t != before).map(|t| (ms_since(t0), t));
    let (first, mut last) = win::poll_until(g.text_wait_ms, ctx.cfg.timing.poll_ms, changed)?;
    while t0.elapsed() < Duration::from_millis(g.text_wait_ms) {
        sleep_ms(ctx.cfg.probes.settle_ms);
        match read() {
            Some(now) if now != last => last = now,
            _ => break,
        }
    }
    Some((first, last))
}

/// One sentence: switch layout, Mic on, the owner speaks, Mic off, then the words must arrive with focus kept.
/// Mic is never clicked unless the layout switched, and once on, its stop click is retried until it lands.
fn one(ctx: &Ctx, uia: &Uia, fw: &FaceWin, app: &Opened, s: &Sentence) -> Result<Value, String> {
    let g = &ctx.cfg.g22;
    let hkl = s
        .layout_id()
        .and_then(langkey::installed_for)
        .ok_or_else(|| format!("layout {} is not installed", s.layout))?;
    if !g25::switch_to(ctx, app, hkl) {
        return Err(format!("Notepad did not switch to layout {}", s.layout));
    }
    let before = uia
        .doc_text(app.hwnd, g.max_chars)
        .ok_or("Notepad's text cannot be read")?;
    keys_ours(app)?;
    fw.press(ctx, ToolButton::Mic)?;
    let t0 = Instant::now();
    let (on_top, under) = bar_seen(ctx, g.record_ms);
    sleep_ms(g.record_ms.saturating_sub(ms_since(t0)));
    let front_rec = keys_ours(app).is_ok();
    let stop = stop_mic(ctx, fw);
    let got = words_in(ctx, uia, app, &before);
    let kept = front_rec && foreground() == app.hwnd;
    let (ms, heard) = got.map_or((None, String::new()), |(ms, t)| {
        (Some(ms), added(&before, &t))
    });
    Ok(json!({
        "layout": s.layout, "bar_on_top": on_top, "bar_under": under,
        "stop_error": stop.as_ref().err(), "stop_to_text_ms": ms, "wer": wer::wer(&s.text, &heard),
        "heard_words": wer::words(&heard).len(), "focus_kept": kept,
        "ok": on_top && stop.is_ok() && ms.is_some() && kept,
    }))
}

/// Every sentence in order, then the starting layout back; Notepad is saved at the end, so closing it asks nothing.
fn drive(ctx: &Ctx, fw: &FaceWin, app: &Opened) -> Result<Value, String> {
    let uia = Uia::new()?;
    probe::front(ctx, app)?;
    let start = langkey::layout_of(app.hwnd);
    let list = &ctx.cfg.g22.sentences;
    let mut all = Vec::new();
    for (i, s) in list.iter().enumerate() {
        eprintln!("sentence {}/{} ({})", i + 1, list.len(), s.layout);
        let mut row = one(ctx, &uia, fw, app, s)
            .unwrap_or_else(|e| json!({ "layout": s.layout, "error": e, "ok": false }));
        row["bar_hid"] = json!(bar_gone(ctx));
        all.push(row);
        sleep_ms(ctx.cfg.g22.pause_ms);
    }
    let restored = start.is_some_and(|h| g25::switch_to(ctx, app, h));
    let saved = keys_ours(app).and_then(|()| readback::read_back(ctx, app).map(drop));
    let pass = all.iter().all(|r| r["ok"] == true);
    Ok(json!({
        "pass": pass, "saved": saved.is_ok(), "layout_restored": restored, "sentences": all,
    }))
}

/// Runs G22 with face `name` in Notepad, the bench for `bench` and `bench-cloud`, or the typing check.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name == g22type::TYPING {
        return g22type::run(ctx);
    }
    featcfg::one_per_language(&ctx.cfg.g22, &ctx.spike.voice)?;
    if name == BENCH || name == g22bench::WITH_CLOUD {
        return g22bench::run(ctx, name);
    }
    fits(&ctx.cfg.g22, &ctx.spike.voice)?;
    let fw = FaceWin::parked(ctx, name)?;
    let doc = probe::open_text(ctx, AppKind::Notepad, "")?;
    let (mut v, notes) = probe::run_on(ctx, doc, |doc| drive(ctx, &fw, &doc.app))?;
    let kept = g22bench::clips(&ctx.cfg.g22.clips_dir()).map_or(0, |c| c.len());
    v["gate"] = json!("G22");
    v["face"] = json!(name);
    v["clips_kept"] = json!(kept);
    v["clean_up"] = json!(notes);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_waits_fit_the_voice_settings() {
        let h = crate::config::load().expect("harness.toml loads");
        let v = spike_core::config::load().expect("spike.toml loads").voice;
        assert!(fits(&h.g22, &v).is_ok());
        let mut g = h.g22.clone();
        g.record_ms = v.record_max_s * MS_PER_S;
        assert!(fits(&g, &v).is_err(), "the recording would stop by itself");
        let mut g = h.g22.clone();
        g.stop_wait_ms = v.record_max_s * MS_PER_S - g.record_ms;
        assert!(
            fits(&g, &v).is_err(),
            "a retried stop click could start a new recording"
        );
        let mut g = h.g22.clone();
        g.text_wait_ms = v.local.timeout_ms;
        assert!(
            fits(&g, &v).is_err(),
            "the harness would give up before the engine"
        );
    }

    #[test]
    fn only_the_new_words_count() {
        assert_eq!(added("", "hello there "), "hello there");
        assert_eq!(added("one two ", "one two three four "), "three four");
        assert_eq!(added("one\r", "one مرحبا\r"), "مرحبا");
        assert_eq!(
            added("gone", "other"),
            "other",
            "a changed start keeps the whole text"
        );
    }
}
