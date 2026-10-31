//! G8's clipboard part (`g8 clip`): the test build's history filled with `[g8_clip] entries` gate-made copies, then
//! rounds in target-window that open the panel, turn pages, search and close. Memory after the fill and with the panel
//! open is judged by `[g8]`, the rounds' growth by `churn_growth_mb`; the fill's and the pictures' cost only report.
#![cfg(windows)]

use serde::Serialize;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::budget::{self, Part};
use crate::clipcopy::{self, ClipSession};
use crate::clippanel::Panel;
use crate::copyend::Ended;
use crate::g8cfg::G8;
use crate::g8churn::{self, MB_DECIMALS};
use crate::g8start::Started;
use crate::g27type::Typist;
use crate::out::say;
use crate::proc::{Memory, Process};
use crate::product::NAME;
use crate::win::sleep_ms;
use crate::{clipfill, clipkeep, g20keys, mouseguard, probe, typebox, winclip};

/// The name that runs G8's clipboard part.
pub const CLIP: &str = "clip";
/// The readings after a settle besides one per judged round: the empty history, the filled one, and before the rounds.
pub const READINGS: usize = 3;

/// One run's memory readings.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Readings {
    /// Before the fill.
    pub empty: Memory,
    /// After the fill and its saves.
    pub filled: Memory,
    /// The most a judged round read with the panel open after its page turns.
    pub open: Memory,
    /// Before the judged rounds, after the warm-up rounds.
    pub before: Memory,
    /// After the last round.
    pub last: Memory,
}

/// The larger of `a` and `b`, each figure on its own.
fn most(a: Memory, b: Memory) -> Memory {
    Memory {
        working_mb: a.working_mb.max(b.working_mb),
        commit_mb: a.commit_mb.max(b.commit_mb),
    }
}

/// The judged parts: the filled and open memory against `[g8] memory_mb` and `commit_mb`, and the rounds' growth
/// against `churn_growth_mb`.
pub(crate) fn parts(g: &G8, r: &Readings) -> Vec<Part> {
    let mut parts = vec![
        Part::at_most("filled_working_mb", Some(r.filled.working_mb), g.memory_mb),
        Part::at_most("filled_commit_mb", Some(r.filled.commit_mb), g.commit_mb),
        Part::at_most("open_working_mb", Some(r.open.working_mb), g.memory_mb),
        Part::at_most("open_commit_mb", Some(r.open.commit_mb), g.commit_mb),
    ];
    let names = ["rounds_working_mb", "rounds_commit_mb"];
    parts.extend(g8churn::growth(names, r.before, r.last, g.churn_growth_mb));
    parts
}

/// The reading line: what the fill and the open panel's pictures added, in MB; readings, never a fail.
pub(crate) fn reading(r: &Readings) -> String {
    let (fill, shown) = (r.filled.less(r.empty), r.open.less(r.filled));
    format!(
        "the fill added {:.MB_DECIMALS$} MB working set and {:.MB_DECIMALS$} MB commit; the open panel {:.MB_DECIMALS$} \
         MB and {:.MB_DECIMALS$} MB; readings, never a fail",
        fill.working_mb, fill.commit_mb, shown.working_mb, shown.commit_mb
    )
}

/// Writes every copy of the fill a gap apart, stopped by the mouse guard.
fn fill(ctx: &Ctx) -> Result<(), String> {
    let g = &ctx.cfg.g8_clip;
    let wait = clipkeep::wait(&ctx.cfg.timing);
    for (n, f) in clipfill::plan(g).into_iter().enumerate() {
        mouseguard::allowed()?;
        clipfill::copy(g, n, f, wait)?;
        sleep_ms(g.copy_gap_ms);
    }
    Ok(())
}

/// One round: the clip key opens the panel, the pages turn, the memory is read, then the search is typed, cleared
/// with Esc, and the clip key shuts the panel.
fn round(ctx: &Ctx, t: &Typist, panel: &Panel, process: &Process) -> Result<Memory, String> {
    let (g, settle) = (&ctx.cfg.g8_clip, ctx.cfg.g20_product.settle_ms);
    g20keys::clip_key(t.app(), panel, true)?;
    for _ in 0..g.pages {
        panel.click(&ctx.cfg.g20_product.page_on)?;
        sleep_ms(settle);
    }
    let open = process.memory()?;
    t.type_in(t.switch(&g.search_lang)?, &g.search)?;
    sleep_ms(settle);
    g20keys::esc(t.app())?;
    sleep_ms(settle);
    g20keys::clip_key(t.app(), panel, false)?;
    Ok(open)
}

/// The warm-up rounds, then the judged ones: the memory before them, the most with the panel open, and after the last.
fn rounds(ctx: &Ctx, t: &Typist, s: &Started) -> Result<[Memory; 3], String> {
    let (g, panel) = (&ctx.cfg.g8_clip, Panel::new(ctx)?);
    let read = || {
        sleep_ms(g.settle_ms);
        s.process.memory()
    };
    for _ in 0..g.warm_rounds {
        round(ctx, t, &panel, &s.process)?;
    }
    let before = read()?;
    let (mut open, mut last) = (None, before);
    for _ in 0..g.rounds {
        let o = round(ctx, t, &panel, &s.process)?;
        open = Some(open.map_or(o, |m| most(m, o)));
        last = read()?;
    }
    Ok([before, open.ok_or("no judged round ran")?, last])
}

/// What a run measured, and how target-window's box went.
struct Run {
    readings: Result<Readings, String>,
    back: bool,
    left: Vec<String>,
}

/// The readings on copy `s`: empty, then filled, then over the rounds in a fresh target-window box.
fn measure(ctx: &Ctx, s: &Started) -> Run {
    let settle = ctx.cfg.g8_clip.settle_ms;
    let read = || {
        sleep_ms(settle);
        s.process.memory()
    };
    let (empty, filled) = match read().and_then(|e| fill(ctx).and_then(|()| read()).map(|f| (e, f)))
    {
        Ok(two) => two,
        Err(e) => {
            return Run {
                readings: Err(e),
                back: true,
                left: Vec::new(),
            };
        }
    };
    let boxed = typebox::in_target(ctx, &[], |t| rounds(ctx, t, s));
    let readings = boxed.got.map(|[before, open, last]| Readings {
        empty,
        filled,
        open,
        before,
        last,
    });
    Run {
        readings,
        back: boxed.back,
        left: boxed.left,
    }
}

/// The result line. It passes when every judged part kept within its limit, the copy quit by itself, target-window's
/// first layout and the test build's settings came back.
fn result(
    ctx: &Ctx,
    run: &Result<Run, String>,
    ended: Option<&Ended>,
    history: &Value,
    back: &Result<(), String>,
) -> Value {
    let readings = run.as_ref().ok().and_then(|r| r.readings.as_ref().ok());
    let judged = readings.map(|r| parts(&ctx.cfg.g8, r));
    let held = judged.as_ref().is_some_and(|p| budget::all_pass(p));
    let box_back = run.as_ref().is_ok_and(|r| r.back);
    let ok = held && ended.is_some_and(Ended::ok) && box_back && back.is_ok();
    let error = match run {
        Err(e) => Some(e.as_str()),
        Ok(r) => r.readings.as_ref().err().map(String::as_str),
    };
    json!({
        "gate": "G8", "part": CLIP, "keyboard": NAME, "entries": ctx.cfg.g8_clip.entries, "memory": readings,
        "parts": judged, "reading": readings.map(reading), "error": error, "copy": ended.map(Ended::line),
        "layout_back": box_back, "clean_up": run.as_ref().map(|r| r.left.clone()).unwrap_or_default(),
        "history": history, "settings_put_back": back.is_ok(), "settings_error": back.as_ref().err(), "pass": ok,
    })
}

/// Runs `g8 clip`, refused under uiAccess, while a product copy runs, or without the worker beside the test build.
fn clip(ctx: &Ctx) -> Result<Value, String> {
    let session = ClipSession::open(ctx)?;
    let since = winclip::now();
    let (run, ended) = match clipcopy::worker(&session.program) {
        Ok(()) => session.with_copy(true, ctx.cfg.g8.warm_ms, |s| Ok(measure(ctx, s))),
        Err(e) => (Err(e), None),
    };
    let prefix = ctx.cfg.g8_clip.prefix.as_str();
    let history = probe::forget_copies(ctx, since, &|t: &str| winclip::has_prefix(t, &[prefix]));
    let back = session.close();
    let v = result(ctx, &run, ended.as_ref(), &history, &back);
    say!("G8 {CLIP}: {}", v["reading"]);
    Ok(v)
}

/// Runs `g8 clip`; none for another name.
pub fn run(ctx: &Ctx, name: &str) -> Option<Result<Value, String>> {
    (name == CLIP).then(|| clip(ctx))
}

#[cfg(test)]
mod tests;
