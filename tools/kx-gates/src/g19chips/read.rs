//! `g19 translate-read` and `g19 translate-beside` (Rulings S and Y): the gate's Notepad document with the long Arabic
//! line, which the gate selects itself; the owner clicks Translate, then Yes. The gate reads only the browser window
//! that opened: its parent, its rights against the launching keyboard's, and whether its address holds the line
//! whole. A verdict the launcher could not exercise says so. Only names, numbers and verdicts print.
#![cfg(windows)]

use std::collections::HashSet;
use std::path::Path;

use kx_platform_windows::uia::Uia;
use kx_platform_windows::window::foreground;
use serde_json::{Value, json};

use crate::apps::{AppKind, Ctx, Opened, same_name};
use crate::g19chips::addr::{self, Shown};
use crate::g19chips::copy::Fresh;
use crate::g19chips::link::site_key;
use crate::g19chips::procs::{self, Proc};
use crate::g19chips::select;
use crate::g19chips::strip::Strip;
use crate::g20prod::copy_line;
use crate::keymap::Reader;
use crate::out::say;
use crate::p10cfg::selection_defaults;
use crate::product::Keyboard;
use crate::workertok::{self, MEDIUM};
use crate::{clipkeep, probe, win};

/// The step on the test build it starts, which has no uiAccess.
pub const READ: &str = "translate-read";
/// The step beside the keyboard the owner started, the uiAccess dev install for the manual check.
pub const BESIDE: &str = "translate-beside";
/// What the owner does while the gate waits.
const ASK: &str = "click Translate on the strip, then Yes if it asks";
/// What a verdict says when the launcher had no uiAccess to pass on.
const NOT_EXERCISED: &str = "not exercised: the launcher has no uiAccess";
/// A verdict that held.
const PASS: &str = "pass";
/// A verdict that did not hold.
const FAIL: &str = "fail";

/// The launcher's program and process, and what held before the owner's turn.
struct Turn<'a> {
    ctx: &'a Ctx<'a>,
    launcher: u32,
    ours: String,
    key: String,
    running: Vec<Proc>,
    windows: HashSet<isize>,
}

/// The uiAccess and integrity verdicts on a browser opened by a launcher with `launcher_ui`: each passes or fails
/// only when the launcher had uiAccess to pass on.
pub fn verdicts(launcher_ui: bool, browser_ui: bool, level: u32) -> (Value, Value) {
    let said = |held: bool| match (launcher_ui, held) {
        (false, _) => NOT_EXERCISED,
        (true, true) => PASS,
        (true, false) => FAIL,
    };
    (json!(said(!browser_ui)), json!(said(level <= MEDIUM)))
}

/// True when the page holds the line whole, no keyboard of ours started the browser, and no verdict of `said`
/// failed; one not exercised fails nothing.
pub fn passed(said: &[&Value], by_us: bool, whole: bool) -> bool {
    whole && !by_us && said.iter().all(|v| v.as_str() != Some(FAIL))
}

/// The reads on the browser that shows `found`: its parent, its rights, and the launcher's.
fn rights(t: &Turn, found: &Shown) -> Result<Value, String> {
    let pid = win::owner(found.hwnd).0;
    let all = procs::now();
    let parent = procs::find(&all, pid)
        .and_then(|p| procs::find(&all, p.parent))
        .map(|p| p.exe.clone());
    let (r, l) = (workertok::rights(pid)?, workertok::rights(t.launcher)?);
    let cold = !procs::launches(&t.running, &all, &t.ctx.cfg.g19_product.browsers).is_empty();
    let by_us = parent.as_deref().is_some_and(|p| same_name(p, &t.ours));
    let (ui, integrity) = verdicts(l.ui_access, r.ui_access, r.level);
    let ok = passed(&[&ui, &integrity], by_us, found.whole);
    Ok(json!({
        "launcher_ui_access": l.ui_access, "launcher_integrity": l.level, "cold_start": cold, "parent": parent,
        "parent_is_ours": by_us, "ui_access": ui, "integrity": integrity, "browser_integrity": r.level,
        "chars": found.chars, "whole": found.whole, "ok": ok,
    }))
}

/// The owner's turn on the gate's document `app`: the line selected, the strip shown, then the page read.
fn turn(t: &Turn, app: &Opened) -> Result<Value, String> {
    let (g, reader) = (&t.ctx.cfg.g19_product, Reader::new()?);
    let uia = Uia::new().map_err(|e| e.to_string())?;
    probe::front(t.ctx, app)?;
    select::all(&uia, app.hwnd)?;
    if Strip::new(t.ctx, t.launcher)?.shows()?.is_none() {
        return Ok(json!({ "ok": false, "error": "no strip showed on the selected line" }));
    }
    let (long, front_before) = (g.long_line(), win::key(foreground()));
    say!("G19: {ASK}; the gate waits up to {} ms", g.owner_wait_ms);
    let found = win::poll_until(g.owner_wait_ms, t.ctx.cfg.timing.poll_ms, || {
        let now = addr::windows(&g.browsers);
        let keys: Vec<isize> = now.iter().map(|w| win::key(*w)).collect();
        let w = addr::pick(&t.windows, front_before, &keys, win::key(foreground()))?;
        let hwnd = now.into_iter().find(|h| win::key(*h) == w)?;
        addr::page(&reader, hwnd, &t.key, &long, g.bar_depth)
    });
    let Some(f) = found else {
        let e = format!("no translate page showed within {} ms", g.owner_wait_ms);
        return Ok(json!({ "ok": false, "error": e }));
    };
    rights(t, &f)
}

/// The gate's document with the long line, the owner's turn beside launcher `launcher`, and the reads; the document
/// closes after, and the page the owner opened stays, as it is the owner's browser.
fn read(ctx: &Ctx, launcher: u32, program: &Path) -> Result<Value, String> {
    let g = &ctx.cfg.g19_product;
    let key =
        site_key(&selection_defaults()?).ok_or("the site's link names no key for its text")?;
    let ours = (program.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let windows = addr::windows(&g.browsers)
        .into_iter()
        .map(win::key)
        .collect();
    let t = Turn {
        ctx,
        launcher,
        ours,
        key,
        running: procs::now(),
        windows,
    };
    let doc = probe::open_text(ctx, AppKind::Notepad, &g.long_line())?;
    let (mut v, notes) = probe::run_on(ctx, doc, |d| turn(&t, &d.app))?;
    v["clean_up"] = json!(notes);
    Ok(v)
}

/// True when `run` passed, its data folder went, and its clean-up left nothing.
pub fn pass_of(run: &Value, removed: bool) -> bool {
    let left = run["clean_up"].as_array().is_some_and(|l| !l.is_empty());
    run["ok"] == true && removed && !left
}

/// The result line of step `name`.
fn line(name: &str, run: &Value, removed: Result<(), String>) -> Value {
    let pass = pass_of(run, removed.is_ok());
    say!("G19 {name}: {}", run["ok"]);
    json!({
        "gate": "G19", "name": name, "run": run, "data_removed": removed.is_ok(), "data_error": removed.err(),
        "pass": pass,
    })
}

/// Runs `translate-read` on the test build it starts, refused under uiAccess or while a product copy runs; the data
/// folder goes at the end.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let fresh = Fresh::open(ctx)?;
    let program = fresh.program.clone();
    let (got, ended) = fresh.with_copy(false, |pid| {
        clipkeep::keep(ctx.cfg, || read(ctx, pid, &program))
    });
    let removed = fresh.close(ended.as_ref());
    Ok(line(READ, &copy_line(got, ended.as_ref()), removed))
}

/// Runs `translate-beside` next to the keyboard the owner started, starting nothing.
pub fn beside(ctx: &Ctx) -> Result<Value, String> {
    let kb = Keyboard::find()?;
    let (pid, program) = (win::owner(kb.hwnd).0, win::program(kb.hwnd)?);
    let got = clipkeep::keep(ctx.cfg, || read(ctx, pid, Path::new(&program)));
    let run = got.unwrap_or_else(|e| json!({ "ok": false, "error": e }));
    Ok(line(BESIDE, &run, Ok(())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_launcher_without_uiaccess_exercises_neither_verdict() {
        let (ui, integrity) = verdicts(false, false, MEDIUM);
        assert_eq!(
            (ui.as_str(), integrity.as_str()),
            (Some(NOT_EXERCISED), Some(NOT_EXERCISED))
        );
        let (ui, integrity) = verdicts(true, false, MEDIUM);
        assert_eq!((ui.as_str(), integrity.as_str()), (Some(PASS), Some(PASS)));
        let (ui, integrity) = verdicts(true, true, MEDIUM + 1);
        assert_eq!((ui.as_str(), integrity.as_str()), (Some(FAIL), Some(FAIL)));
    }

    #[test]
    fn the_read_passes_on_the_whole_line_and_every_verdict_it_exercised() {
        let read = |launcher, browser, level, by_us, whole| {
            let (ui, integrity) = verdicts(launcher, browser, level);
            passed(&[&ui, &integrity], by_us, whole)
        };
        assert!(read(true, false, MEDIUM, false, true));
        assert!(
            read(false, true, MEDIUM + 1, false, true),
            "nothing to inherit"
        );
        assert!(!read(true, true, MEDIUM, false, true), "uiAccess passed on");
        assert!(!read(true, false, MEDIUM + 1, false, true), "above Medium");
        assert!(
            !read(true, false, MEDIUM, true, true),
            "our keyboard is its parent"
        );
        assert!(!read(true, false, MEDIUM, false, false), "the line cut");
    }

    #[test]
    fn a_window_or_file_left_behind_fails_the_step() {
        assert!(pass_of(&json!({ "ok": true, "clean_up": [] }), true));
        assert!(!pass_of(
            &json!({ "ok": true, "clean_up": ["left open: Notepad"] }),
            true
        ));
        assert!(
            !pass_of(&json!({ "ok": true, "clean_up": [] }), false),
            "the data folder"
        );
        assert!(!pass_of(&json!({ "ok": false }), true));
    }

    #[test]
    fn the_owner_is_asked_only_to_click_never_to_select_or_drag() {
        assert!(ASK.contains("Translate") && ASK.contains("Yes"), "{ASK}");
        assert!(!ASK.contains("select") && !ASK.contains("drag"), "{ASK}");
    }
}
