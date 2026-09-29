//! G17 Grab: a still hold grabs, one click drops. Explorer moves a file and Esc cancels; text apps select.
#![cfg(windows)]

use serde_json::{Value, json};
use spike_core::hold::Mode;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LBUTTON;

use crate::apps::{AppKind, Ctx};
use crate::assist::Assist;
use crate::probe::{self, Doc};
use crate::simuser::{self, is_up};
use crate::win::{self, sleep_ms};
use crate::{clipkeep, winclip};
use spike_core::uia::Uia;

/// The apps G17 probes.
pub const APPS: [AppKind; 4] = [
    AppKind::Explorer,
    AppKind::Notepad,
    AppKind::Chrome,
    AppKind::Word,
];

/// One case's line; it passes only if the left button is up after it.
fn line(case: &str, ok: bool, note: Value) -> Value {
    let up = is_up(VK_LBUTTON.0);
    json!({ "case": case, "ok": ok && up, "left_up": up, "note": note })
}

/// Grabs the move file and drops it on the drop folder; passes when it moved on disk.
fn move_file(ctx: &Ctx, uia: &Uia, assist: &Assist, doc: &Doc) -> Result<Value, String> {
    let (pr, t, app) = (&ctx.cfg.probes, &ctx.cfg.timing, &doc.app);
    let (_, from) = probe::item(ctx, uia, app, &pr.move_file)?;
    let (_, to) = probe::item(ctx, uia, app, &pr.drop_dir)?;
    assist.set_mode(Mode::Grab)?;
    simuser::grab(ctx, app, from, to)?;
    simuser::click(ctx, app, to)?;
    let was = doc.path.join(&pr.move_file);
    let now = doc.path.join(&pr.drop_dir).join(&pr.move_file);
    let moved = || (now.exists() && !was.exists()).then_some(());
    let ok = win::poll_until(t.read_wait_ms, t.poll_ms, moved).is_some();
    Ok(line("explorer_move", ok, json!({ "moved": ok })))
}

/// Grabs the keep file over the drop folder, then taps Esc; passes when a carry was live and the file stayed.
fn esc_keeps(ctx: &Ctx, uia: &Uia, assist: &Assist, doc: &Doc) -> Result<Value, String> {
    let (pr, app) = (&ctx.cfg.probes, &doc.app);
    let (_, from) = probe::item(ctx, uia, app, &pr.keep_file)?;
    let (_, to) = probe::item(ctx, uia, app, &pr.drop_dir)?;
    assist.set_mode(Mode::Grab)?;
    simuser::grab(ctx, app, from, to)?;
    let carried = !is_up(VK_LBUTTON.0);
    simuser::esc(app)?;
    sleep_ms(pr.settle_ms);
    let stayed = doc.path.join(&pr.keep_file).exists()
        && !doc.path.join(&pr.drop_dir).join(&pr.keep_file).exists();
    let note = json!({ "carried": carried, "stayed": stayed });
    Ok(line("explorer_esc", carried && stayed, note))
}

/// Grabs at the start of the first line and drops at its middle; passes when a part of the text copies.
fn select(ctx: &Ctx, uia: &Uia, assist: &Assist, doc: &Doc) -> Result<Value, String> {
    let app = &doc.app;
    let (start, mid) = probe::text_points(ctx, uia, app)?;
    assist.set_mode(Mode::Grab)?;
    simuser::grab(ctx, app, start, mid)?;
    simuser::click(ctx, app, mid)?;
    let (ok, note) = probe::copied(ctx, app);
    Ok(line("select", ok, note))
}

/// Runs the app's cases with Grab on the simulated user's input.
fn drive(ctx: &Ctx, uia: &Uia, doc: &Doc) -> Result<(Vec<Value>, Vec<String>), String> {
    probe::front(ctx, &doc.app)?;
    let assist = simuser::start_assist(ctx, Vec::new())?;
    let cases = match doc.app.kind {
        AppKind::Explorer => vec![
            move_file(ctx, uia, &assist, doc)?,
            esc_keeps(ctx, uia, &assist, doc)?,
        ],
        _ => vec![select(ctx, uia, &assist, doc)?],
    };
    Ok((cases, assist.stop()?.errors))
}

/// Runs G17 on the app called `name`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    let uia = Uia::new()?;
    let pr = &ctx.cfg.probes;
    let doc = match kind {
        AppKind::Explorer => probe::open_folder(
            ctx,
            &[pr.move_file.clone(), pr.keep_file.clone()],
            std::slice::from_ref(&pr.drop_dir),
        )?,
        _ => probe::open_text(ctx, kind, &pr.text)?,
    };
    println!("G17 {name}: {}", win::describe(doc.app.hwnd));
    let since = winclip::now();
    let run = |d: &Doc| clipkeep::keep(ctx.cfg, || drive(ctx, &uia, d));
    let result = probe::run_on(ctx, doc, run);
    // Clean the history even when the run failed.
    let history = probe::forget_copies(ctx, since, &|t| probe::probe_copy(ctx, t));
    let ((cases, errors), notes) =
        result.map_err(|e| format!("{e}; history removed: {history}"))?;
    let pass = cases.iter().all(|c| c["ok"] == json!(true)) && errors.is_empty();
    println!("G17 {name}: pass {pass}; {}", json!(cases));
    Ok(json!({
        "gate": "G17", "app": name, "pass": pass, "cases": cases,
        "errors": errors, "clean_up": notes, "history_removed": history,
    }))
}
