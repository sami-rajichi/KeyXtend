//! `g19 translate-read` (Ruling S): the test build, plain, on a fresh data folder, and the gate's Notepad document with
//! the long Arabic line; the owner selects it, clicks Translate and Yes. The gate only reads the page that opened: its
//! browser's parent, integrity and uiAccess, and whether the address bar holds the line whole. Only numbers print.
#![cfg(windows)]

use std::path::Path;

use serde_json::{Value, json};

use crate::apps::{AppKind, Ctx, same_name};
use crate::g19chips::addr::{self, Shown};
use crate::g19chips::copy::Fresh;
use crate::g19chips::link::site_key;
use crate::g19chips::procs::{self, Proc};
use crate::g20prod::copy_line;
use crate::keymap::Reader;
use crate::out::say;
use crate::p10cfg::selection_defaults;
use crate::workertok::{self, MEDIUM};
use crate::{clipkeep, probe, win};

/// The step's name on the command line.
pub const READ: &str = "translate-read";
/// What the owner does while the gate waits.
const ASK: &str =
    "select the whole Arabic line in the gate's Notepad document, click Translate, then Yes";

/// The reads on the browser that shows `found`: its parent, its rights, whether it started anew since `running`, and
/// the line's characters; it passes with no uiAccess, at most Medium, no parent of ours, and the line whole.
fn rights(ctx: &Ctx, found: &Shown, running: &[Proc], ours: &str) -> Result<Value, String> {
    let pid = win::owner(found.hwnd).0;
    let all = procs::now();
    let parent = procs::find(&all, pid)
        .and_then(|p| procs::find(&all, p.parent))
        .map(|p| p.exe.clone());
    let r = workertok::rights(pid)?;
    let cold = !procs::launches(running, &all, &ctx.cfg.g19_product.browsers).is_empty();
    let by_us = parent.as_deref().is_some_and(|p| same_name(p, ours));
    let ok = !r.ui_access && r.level <= MEDIUM && !by_us && found.whole;
    Ok(json!({
        "parent": parent, "parent_is_ours": by_us, "integrity": r.level, "ui_access": r.ui_access,
        "cold_start": cold, "chars": found.chars, "whole": found.whole, "ok": ok,
    }))
}

/// Waits up to `owner_wait_ms` for the page, then reads it.
fn wait_page(ctx: &Ctx, key: &str, running: &[Proc], ours: &str) -> Result<Value, String> {
    let (g, reader) = (&ctx.cfg.g19_product, Reader::new()?);
    let long = g.long_line();
    let found = win::poll_until(g.owner_wait_ms, g.settle_ms, || {
        addr::page(&reader, &g.browsers, key, &long)
    });
    match found {
        Some(f) => rights(ctx, &f, running, ours),
        None => Ok(
            json!({ "ok": false, "error": format!("no translate page showed within {} ms", g.owner_wait_ms) }),
        ),
    }
}

/// The gate's document with the long line, the owner's turn, and the reads; the document closes after.
fn read(ctx: &Ctx, program: &Path) -> Result<Value, String> {
    let g = &ctx.cfg.g19_product;
    let key =
        site_key(&selection_defaults()?).ok_or("the site's link names no key for its text")?;
    let ours = program
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let running = procs::now();
    let doc = probe::open_text(ctx, AppKind::Notepad, &g.long_line())?;
    say!(
        "G19 {READ}: {ASK}; the gate waits up to {} ms",
        g.owner_wait_ms
    );
    let (mut v, notes) = probe::run_on(ctx, doc, |_| wait_page(ctx, &key, &running, &ours))?;
    v["clean_up"] = json!(notes);
    Ok(v)
}

/// Runs `translate-read`, refused under uiAccess or while a product copy runs; the data folder goes at the end. The
/// page the owner opened stays open, as it is the owner's browser.
pub fn run(ctx: &Ctx) -> Result<Value, String> {
    let fresh = Fresh::open(ctx)?;
    let program = fresh.program.clone();
    let (got, ended) = fresh.with_copy(false, |_| clipkeep::keep(ctx.cfg, || read(ctx, &program)));
    let removed = fresh.close(ended.as_ref());
    let run = copy_line(got, ended.as_ref());
    let pass = run["ok"] == true && removed.is_ok();
    say!("G19 {READ}: {}", run["ok"]);
    Ok(json!({
        "gate": "G19", "name": READ, "run": run, "data_removed": removed.is_ok(), "data_error": removed.err(),
        "pass": pass,
    }))
}
