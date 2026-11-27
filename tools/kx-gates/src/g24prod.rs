//! G24 on the product (`g24 keyxtend`): the test build, its keyboard as a bubble then hidden, snipping and recording
//! target-window by clicks only and judged by sizes, pixel shares and lengths; `g24 trap` runs the trap part alone.
//! Only numbers and our own titles are logged, and nothing the gate made stays.
#![cfg(windows)]

mod clip;
mod clock;
mod copy;
mod copyset;
mod hdrop;
mod judge;
mod low;
mod mp4;
mod order;
mod pic;
mod render;
mod scene;
mod shots;
mod snip;
mod trap;
mod video;
mod watch;
mod wins;

use std::path::{Path, PathBuf};

use kx_module_api::ports::SnipKeyboard;
use kx_platform_windows::worker::WORKER_EXE;
use serde_json::{Value, json};

use crate::apps::{self, Ctx, Opened};
use crate::copyend::{self, Ended};
use crate::freshcopy::Fresh;
use crate::g20prod::copy_line;
use crate::g25::part;
use crate::out::say;
use crate::p12cfg::G24;
use crate::product::NAME;
use crate::win::{poll_until, sleep_ms};
use crate::winpanel::judged_ok;
use crate::{clipkeep, launch, probe, simuser, winclip, workertok};
use copy::Dirs;
use scene::Scene;
use snip::{Copied, PictureCopies, Snip};

/// The gate's name in its result.
const GATE: &str = "G24";
/// The keyboard's ways the copies run with, in order.
const MODES: [SnipKeyboard; crate::p12cfg::COPIES] = [SnipKeyboard::Bubble, SnipKeyboard::Hidden];
/// Why a copy did not start: the one before may still run.
const STILL_RUNS: &str = "not started: the copy before may still run";

/// Which G24 runs: every part on both copies, or the trap part alone (ADR-0059).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Run {
    All,
    Trap,
}

impl Run {
    /// Its name on the command line and in the result.
    fn name(self) -> &'static str {
        match self {
            Self::All => NAME,
            Self::Trap => crate::g24::TRAP,
        }
    }

    /// The keyboard ways its copies take, in order.
    fn ways(self) -> Vec<SnipKeyboard> {
        let takes = |&w: &SnipKeyboard| self == Self::All || trap::taps(w);
        MODES.into_iter().filter(takes).collect()
    }

    /// The part names of its copy with the keyboard `way`.
    fn names(self, way: SnipKeyboard) -> &'static [&'static str] {
        match self {
            Self::All => copy::names(way),
            Self::Trap => &trap::NAMES,
        }
    }
}

/// True when each judged part of `v` named in `names` passed; a skipped one neither fails nor passes.
fn parts_ok(v: &Value, names: &[&str]) -> bool {
    judged_ok(names.iter().map(|n| &v[*n]))
}

/// True when every copy of `run` ran and passed, their data folder went, and the clean-up left nothing.
fn passed(copies: &[Value], run: Run, removed: bool, left: &[String]) -> bool {
    let all = copies.len() == run.ways().len() && copies.iter().all(|c| c["ok"] == true);
    all && removed && left.is_empty()
}

/// The leftover line for the `pids` of workers that still ran once the copies ended, by count only; `None` for none.
fn workers_left(pids: &[u32]) -> Option<String> {
    (!pids.is_empty()).then(|| {
        format!(
            "{} {WORKER_EXE} still ran after the copies ended",
            pids.len()
        )
    })
}

/// The folder the copy's workers run from: its `program`'s.
fn workers_dir(program: &Path) -> PathBuf {
    program.parent().map(Path::to_path_buf).unwrap_or_default()
}

/// Waits up to `[g24] gone_ms` for the copy's workers to end, so none holds a video in the data folder: a leftover line
/// for those still running.
fn workers_gone(ctx: &Ctx, fresh: &Fresh) -> Option<String> {
    let dir = workers_dir(&fresh.program);
    let still = || workertok::beside(&workertok::processes(), &dir);
    let gone = || still().is_empty().then_some(());
    let _ = poll_until(ctx.cfg.g24.gone_ms, ctx.cfg.timing.poll_ms, gone);
    workers_left(&still())
}

/// `run` on a ready copy `pid` with the keyboard `way`, target-window brought to the front first.
fn drive(
    ctx: &Ctx,
    app: &Opened,
    (pid, way, run): (u32, SnipKeyboard, Run),
    (dirs, copied): (&Dirs, &PictureCopies),
) -> Result<Value, String> {
    probe::front(ctx, app)?;
    let s = Snip::new(ctx, (pid, way), app, copied)?;
    let sc = Scene::read(&s)?;
    let mut v = match (run, way) {
        (Run::Trap, _) => trap::run(&s, &sc, dirs.workers),
        (Run::All, SnipKeyboard::Bubble) => copy::bubble(&s, &sc, dirs),
        _ => copy::hidden(&s, &sc, dirs),
    };
    v["way"] = json!(way);
    v["dpi"] = json!(sc.dpi);
    v["ok"] = json!(parts_ok(&v, run.names(way)));
    Ok(v)
}

/// One copy of `run` with the keyboard `way`, on its settings, its picture Copies noted in `copied`: what it gave and
/// how it went.
fn one(
    (ctx, fresh, app): (&Ctx, &Fresh, &Opened),
    (way, run): (SnipKeyboard, Run),
    copied: &PictureCopies,
) -> (Result<Value, String>, Option<Ended>) {
    let written = copyset::table(way).and_then(|t| fresh.write(&t));
    let workers = workers_dir(&fresh.program);
    let dirs = Dirs {
        data: fresh.data(),
        workers: &workers,
    };
    match written {
        Ok(()) => fresh.with_copy((false, ctx.cfg.g24.ready_ms), |pid| {
            drive(ctx, app, (pid, way, run), (&dirs, copied))
        }),
        Err(e) => (Err(e), None),
    }
}

/// The copies of `run` on `app`, each only once the one before ended: their parts, and how the last one went.
fn copies(
    (ctx, fresh, app): (&Ctx, &Fresh, &Opened),
    copied: &PictureCopies,
    run: Run,
) -> (Vec<Value>, Option<Ended>) {
    let (mut parts, mut last) = (Vec::new(), None::<Ended>);
    for way in run.ways() {
        if !copyend::may_follow(last.as_ref().map(|e| e.gone)) {
            parts.push(copy_line(Err(STILL_RUNS.to_string()), None));
            continue;
        }
        let (got, ended) = one((ctx, fresh, app), (way, run), copied);
        parts.push(copy_line(got, ended.as_ref()));
        last = ended.or(last);
    }
    (parts, last)
}

/// Target-window parked at the top left, in front, holding `[g24] text`.
fn ready(ctx: &Ctx, app: &Opened) -> Result<(), String> {
    launch::park_target(app.hwnd)?;
    probe::front(ctx, app)?;
    simuser::keys_ours(app)?;
    crate::keys::text(&ctx.cfg.g24.text)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    Ok(())
}

/// Starts target-window, runs the copies of `run` on it and closes it: their parts, how the last went, the clean-up.
fn on_target(
    ctx: &Ctx,
    fresh: &Fresh,
    (left, copied, run): (&mut Vec<String>, &PictureCopies, Run),
) -> (Vec<Value>, Option<Ended>) {
    let app = match launch::start_target(ctx) {
        Ok(app) => app,
        Err(e) => return (vec![part(Err(e))], None),
    };
    let got = match ready(ctx, &app) {
        Ok(()) => copies((ctx, fresh, &app), copied, run),
        Err(e) => (vec![part(Err(e))], None),
    };
    left.extend(apps::clean_up(ctx, app));
    got
}

/// Runs G24 on the product, or with `Run::Trap` its trap part, refused under uiAccess or while a product copy runs.
/// The owner's clipboard comes back, the full run's pictures leave Windows' history, and the data folder goes once
/// the copy's workers ended.
pub fn run(ctx: &Ctx, run: Run) -> Result<Value, String> {
    let fresh = Fresh::open(ctx, GATE)?;
    let program = fresh.program.display().to_string();
    let (pictures, mut left, mut got) = (PictureCopies::default(), Vec::new(), (Vec::new(), None));
    let body = || {
        got = on_target(ctx, &fresh, (&mut left, &pictures, run));
        Ok(())
    };
    // The trap part copies nothing, so it writes the clipboard back only if something changed it.
    let kept = match run {
        Run::All => clipkeep::keep(ctx.cfg, body),
        Run::Trap => clipkeep::keep_changed(ctx.cfg, body),
    };
    let (mut copies, ended) = got;
    copies.extend(kept.err().map(|e| part(Err(e))));
    let history = (run == Run::All).then(|| forget(&ctx.cfg.g24, &pictures.borrow()));
    left.extend(workers_gone(ctx, &fresh));
    let removed = fresh.close(ended.as_ref());
    let forgot = history.as_ref().is_none_or(|h| h["ok"] == true);
    let pass = passed(&copies, run, removed.is_ok(), &left) && forgot;
    let note = judge::skipped_note(&copies).map_or_else(String::new, |n| format!(" ({n})"));
    say!("{GATE} {}: {pass}{note}", run.name());
    Ok(json!({
        "gate": GATE, "name": run.name(), "program": program, "copies": copies, "clean_up": left, "history": history,
        "data_removed": removed.is_ok(), "data_error": removed.err(), "pass": pass,
    }))
}

/// The gate's own Copy pictures out of Windows' history, each span's end padded by `[g24] history_pad_ms` short of the
/// next: whether every picture history keeps, up to `history_item_mb`, went.
fn forget(g: &G24, copied: &[Copied]) -> Value {
    let spans: Vec<_> = copied.iter().map(|c| c.span).collect();
    let spans = winclip::padded(&spans, g.history_pad_ms);
    let bytes: Vec<_> = copied.iter().map(|c| c.bytes).collect();
    match winclip::forget_pictures(&spans) {
        Ok(went) => judge::history(went.as_deref(), &bytes, f64::from(g.history_item_mb)),
        Err(e) => json!({ "error": e, "ok": false }),
    }
}

#[cfg(test)]
mod tests;
