//! G17 Grab on the product's test build: a click picks up, one click drops. Explorer moves a file and the keyboard's
//! Esc key cancels; text apps select.
#![cfg(windows)]

use kx_mod_keyboard::{Assist, Role};
use kx_mod_mouse::ESC;
use serde_json::{Value, json};
use spike_core::uia::Uia;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_LBUTTON;

use crate::apps::{AppKind, Ctx};
use crate::g5::RCLICK;
use crate::out::say;
use crate::probe::{self, Doc};
use crate::product::Keyboard;
use crate::simuser::{self, is_up};
use crate::testbuild::TestBuild;
use crate::win;
use crate::{board, clipkeep, winclip};

/// The apps G17 probes.
pub const APPS: [AppKind; 4] = [
    AppKind::Explorer,
    AppKind::Notepad,
    AppKind::Chrome,
    AppKind::Word,
];
/// The key that turns Grab on.
const GRAB: Role = Role::Assist(Assist::Grab);

/// The id of the board's key that sends the engine's Esc.
fn esc_id() -> Result<String, String> {
    let found = board::roles()?
        .into_iter()
        .find(|(.., scan)| *scan == Some(ESC));
    found
        .map(|(id, ..)| id)
        .ok_or_else(|| "the product's board has no Esc key".to_string())
}

/// One case's line; it passes only if the left button is up after it. Its note says whether the Grab key read off
/// afterwards, which is not judged.
fn line(build: &TestBuild, case: &str, ok: bool, mut note: Value) -> Value {
    let up = is_up(VK_LBUTTON.0);
    let grab_off = build.reads(GRAB, false).is_ok();
    if let Some(fields) = note.as_object_mut() {
        fields.insert("grab_off".to_string(), json!(grab_off));
    }
    json!({ "case": case, "ok": ok && up, "left_up": up, "note": note })
}

/// Grabs the move file and drops it on the drop folder; passes when it moved on disk.
fn move_file(ctx: &Ctx, uia: &Uia, build: &TestBuild, doc: &Doc) -> Result<Value, String> {
    let (pr, t, app) = (&ctx.cfg.probes, &ctx.cfg.timing, &doc.app);
    let (_, from) = probe::item(ctx, uia, app, &pr.move_file)?;
    let (_, to) = probe::item(ctx, uia, app, &pr.drop_dir)?;
    build.set(GRAB, true)?;
    simuser::grab(ctx, app, from, to)?;
    simuser::click(ctx, app, to)?;
    let was = doc.path.join(&pr.move_file);
    let now = doc.path.join(&pr.drop_dir).join(&pr.move_file);
    let moved = || (now.exists() && !was.exists()).then_some(());
    let ok = win::poll_until(t.read_wait_ms, t.poll_ms, moved).is_some();
    Ok(line(build, "explorer_move", ok, json!({ "moved": ok })))
}

/// Grabs the keep file over the drop folder, then clicks the keyboard's Esc key; passes when a carry was live and the
/// file stayed.
fn esc_keeps(ctx: &Ctx, uia: &Uia, build: &TestBuild, doc: &Doc) -> Result<Value, String> {
    let (pr, app) = (&ctx.cfg.probes, &doc.app);
    let (_, from) = probe::item(ctx, uia, app, &pr.keep_file)?;
    let (_, to) = probe::item(ctx, uia, app, &pr.drop_dir)?;
    let (keyboard, esc) = (Keyboard::find()?, esc_id()?);
    build.set(GRAB, true)?;
    simuser::grab(ctx, app, from, to)?;
    let carried = !is_up(VK_LBUTTON.0);
    simuser::click_key(ctx, &keyboard, &esc)?;
    let stayed = doc.path.join(&pr.keep_file).exists()
        && !doc.path.join(&pr.drop_dir).join(&pr.keep_file).exists();
    let note = json!({ "carried": carried, "stayed": stayed });
    Ok(line(build, "explorer_esc", carried && stayed, note))
}

/// Grabs at the start of the first line and drops at its middle; passes when a part of the text copies.
fn select(ctx: &Ctx, uia: &Uia, build: &TestBuild, doc: &Doc) -> Result<Value, String> {
    let app = &doc.app;
    let (start, mid) = probe::text_points(ctx, uia, app)?;
    build.set(GRAB, true)?;
    simuser::grab(ctx, app, start, mid)?;
    simuser::click(ctx, app, mid)?;
    let (ok, note) = probe::copied(ctx, app);
    Ok(line(build, "select", ok, note))
}

/// Runs the app's cases, each turning Grab on with the test build's key.
fn drive(ctx: &Ctx, uia: &Uia, build: &TestBuild, doc: &Doc) -> Result<Vec<Value>, String> {
    probe::front(ctx, &doc.app)?;
    Ok(match doc.app.kind {
        AppKind::Explorer => vec![
            move_file(ctx, uia, build, doc)?,
            esc_keeps(ctx, uia, build, doc)?,
        ],
        _ => vec![select(ctx, uia, build, doc)?],
    })
}

/// Opens the app's probe document: Explorer's folder with the move and keep files, else the probe text.
fn open(ctx: &Ctx, kind: AppKind) -> Result<Doc, String> {
    let pr = &ctx.cfg.probes;
    match kind {
        AppKind::Explorer => probe::open_folder(
            ctx,
            &[pr.move_file.clone(), pr.keep_file.clone()],
            std::slice::from_ref(&pr.drop_dir),
        ),
        _ => probe::open_text(ctx, kind, &pr.text),
    }
}

/// Runs G17 on the app called `name`, under the test build with Right-click off, so each press picks up at once; it
/// passes when every case did and the test build quit.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    let uia = Uia::new()?;
    let build = TestBuild::open(ctx, None)?;
    build.set(RCLICK, false)?;
    let doc = open(ctx, kind)?;
    say!("G17 {name}: {}", win::describe(doc.app.hwnd));
    let since = winclip::now();
    let run = |d: &Doc| clipkeep::keep(ctx.cfg, || drive(ctx, &uia, &build, d));
    let result = probe::run_on(ctx, doc, run);
    // Clean the history even when the run failed.
    let history = probe::forget_copies(ctx, since, &|t| probe::probe_copy(ctx, t));
    let closed = build.close();
    let (cases, notes) = result.map_err(|e| {
        format!(
            "{e}; history removed: {history}; test build: {}",
            closed.json()
        )
    })?;
    let pass = closed.pass(cases.iter().all(|c| c["ok"] == json!(true)));
    say!("G17 {name}: pass {pass}; {}", json!(cases));
    Ok(json!({
        "gate": "G17", "app": name, "pass": pass, "cases": cases, "clean_up": notes, "history_removed": history,
        "test_build": closed.json(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_esc_key_is_the_boards_key_that_sends_the_engines_esc() {
        assert_eq!(esc_id(), Ok("esc".to_string()));
    }
}
