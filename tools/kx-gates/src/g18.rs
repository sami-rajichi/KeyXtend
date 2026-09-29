//! G18 modifier+click: a latched Shift or Ctrl changes the next click, then lets go.
#![cfg(windows)]

use serde_json::{Value, json};
use spike_core::hold::Mode;
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_CONTROL, VK_SHIFT};

use crate::apps::{AppKind, Ctx};
use crate::assist::Assist;
use crate::out::say;
use crate::probe::{self, Doc};
use crate::simuser::{self, is_up};
use crate::win;
use crate::{clipkeep, winclip};
use spike_core::uia::{self, Uia};

/// The apps G18 probes.
pub const APPS: [AppKind; 3] = [AppKind::Notepad, AppKind::Chrome, AppKind::Explorer];

/// One case's line; it passes only if the modifier `vk` is up after it.
fn line(case: &str, vk: u16, ok: bool, note: &Value) -> Value {
    let up = is_up(vk);
    json!({ "case": case, "ok": ok && up, "key_up": up, "note": note })
}

/// Clicks at the line start, latches Shift, clicks at its middle; passes when a part of the text copies.
fn shift(ctx: &Ctx, uia: &Uia, assist: &Assist, doc: &Doc) -> Result<Value, String> {
    let app = &doc.app;
    let (start, mid) = probe::text_points(ctx, uia, app)?;
    simuser::click(ctx, app, start)?;
    assist.latch(VK_SHIFT.0)?;
    simuser::click(ctx, app, mid)?;
    let (ok, note) = probe::copied(ctx, app);
    Ok(line("shift_click", VK_SHIFT.0, ok, &note))
}

/// Clicks one file, latches Ctrl, clicks the other; passes when both are selected.
fn ctrl(ctx: &Ctx, uia: &Uia, assist: &Assist, doc: &Doc) -> Result<Value, String> {
    let ([one, two], app) = (&ctx.cfg.probes.ctrl_files, &doc.app);
    let (a, pa) = probe::item(ctx, uia, app, one)?;
    let (b, pb) = probe::item(ctx, uia, app, two)?;
    simuser::click(ctx, app, pa)?;
    assist.latch(VK_CONTROL.0)?;
    simuser::click(ctx, app, pb)?;
    let both = uia::selected(&a).and_then(|x| Ok(x && uia::selected(&b)?));
    let ok = both.as_ref().is_ok_and(|&s| s);
    let note = json!({ "both_selected": both.map_or_else(|e| json!(e), |s| json!(s)) });
    Ok(line("ctrl_click", VK_CONTROL.0, ok, &note))
}

/// Runs the app's case with Right-click on, so each click goes through the held-back path.
fn drive(ctx: &Ctx, uia: &Uia, doc: &Doc) -> Result<(Value, Vec<String>), String> {
    probe::front(ctx, &doc.app)?;
    let assist = simuser::start_assist(ctx, Vec::new())?;
    assist.set_mode(Mode::RightClick)?;
    let case = match doc.app.kind {
        AppKind::Explorer => ctrl(ctx, uia, &assist, doc)?,
        _ => shift(ctx, uia, &assist, doc)?,
    };
    Ok((case, assist.stop()?.errors))
}

/// Runs G18 on the app called `name`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    let uia = Uia::new()?;
    let pr = &ctx.cfg.probes;
    let doc = match kind {
        AppKind::Explorer => probe::open_folder(ctx, &pr.ctrl_files, &[])?,
        _ => probe::open_text(ctx, kind, &pr.text)?,
    };
    say!("G18 {name}: {}", win::describe(doc.app.hwnd));
    let since = winclip::now();
    let run = |d: &Doc| clipkeep::keep(ctx.cfg, || drive(ctx, &uia, d));
    let result = probe::run_on(ctx, doc, run);
    // Clean the history even when the run failed.
    let history = probe::forget_copies(ctx, since, &|t| probe::probe_copy(ctx, t));
    let ((case, errors), notes) = result.map_err(|e| format!("{e}; history removed: {history}"))?;
    let pass = case["ok"] == json!(true) && errors.is_empty();
    say!("G18 {name}: pass {pass}; {case}");
    Ok(json!({
        "gate": "G18", "app": name, "pass": pass, "case": case,
        "errors": errors, "clean_up": notes, "history_removed": history,
    }))
}
