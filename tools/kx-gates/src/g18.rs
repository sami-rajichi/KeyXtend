//! G18 modifier+click on the product's test build: a Shift or Ctrl latched on its keyboard changes the next click, then
//! lets go, and the latch key reads off.
#![cfg(windows)]

use kx_mod_keyboard::{Assist, KeyboardSettings, Latch, Modifiers, Role};
use serde_json::{Value, json};
use spike_core::uia::{self, Uia};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_CONTROL, VK_SHIFT};

use crate::apps::{AppKind, Ctx};
use crate::out::say;
use crate::probe::{self, Doc};
use crate::simuser::{self, is_up};
use crate::testbuild::TestBuild;
use crate::win;
use crate::{clipkeep, winclip};

/// The apps G18 probes.
pub const APPS: [AppKind; 3] = [AppKind::Notepad, AppKind::Chrome, AppKind::Explorer];
/// The key that turns Right-click on, so each click takes the held-back path.
const RCLICK: Role = Role::Assist(Assist::RClick);
/// The Shift latch key.
const SHIFT: Role = Role::Latch(Latch::Shift);
/// The Ctrl latch key.
const CTRL: Role = Role::Latch(Latch::Ctrl);

/// Refuses a keyboard whose latches stay after a click, as each case checks that its latch lets go.
fn latches_release(s: &KeyboardSettings) -> Result<(), String> {
    match s.modifiers {
        Modifiers::Release => Ok(()),
        Modifiers::Stay => Err(
            "G18 needs the test build's [keyboard] modifiers to release after the next key, not stay".to_string(),
        ),
    }
}

/// One case's line; it passes only when its check passed, the modifier `vk` is up and its latch key read off.
fn line((case, vk): (&str, u16), ok: bool, latch_off: bool, note: &Value) -> Value {
    let up = is_up(vk);
    json!({ "case": case, "ok": ok && up && latch_off, "key_up": up, "latch_off": latch_off, "note": note })
}

/// Clicks at the line start, latches Shift, clicks at its middle; passes when a part of the text copies.
fn shift(ctx: &Ctx, uia: &Uia, build: &TestBuild, doc: &Doc) -> Result<Value, String> {
    let app = &doc.app;
    let (start, mid) = probe::text_points(ctx, uia, app)?;
    simuser::click(ctx, app, start)?;
    build.set(SHIFT, true)?;
    simuser::click(ctx, app, mid)?;
    let latch_off = build.reads(SHIFT, false).is_ok();
    let (ok, note) = probe::copied(ctx, app);
    Ok(line(("shift_click", VK_SHIFT.0), ok, latch_off, &note))
}

/// Clicks one file, latches Ctrl, clicks the other; passes when both are selected.
fn ctrl(ctx: &Ctx, uia: &Uia, build: &TestBuild, doc: &Doc) -> Result<Value, String> {
    let ([one, two], app) = (&ctx.cfg.probes.ctrl_files, &doc.app);
    let (a, pa) = probe::item(ctx, uia, app, one)?;
    let (b, pb) = probe::item(ctx, uia, app, two)?;
    simuser::click(ctx, app, pa)?;
    build.set(CTRL, true)?;
    simuser::click(ctx, app, pb)?;
    let latch_off = build.reads(CTRL, false).is_ok();
    let both = uia::selected(&a).and_then(|x| Ok(x && uia::selected(&b)?));
    let ok = both.as_ref().is_ok_and(|&s| s);
    let note = json!({ "both_selected": both.map_or_else(|e| json!(e), |s| json!(s)) });
    Ok(line(("ctrl_click", VK_CONTROL.0), ok, latch_off, &note))
}

/// Runs the app's case with Right-click on.
fn drive(ctx: &Ctx, uia: &Uia, build: &TestBuild, doc: &Doc) -> Result<Value, String> {
    probe::front(ctx, &doc.app)?;
    build.set(RCLICK, true)?;
    match doc.app.kind {
        AppKind::Explorer => ctrl(ctx, uia, build, doc),
        _ => shift(ctx, uia, build, doc),
    }
}

/// Runs G18 on the app called `name`, under the test build, and turns Right-click off after it.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    let uia = Uia::new()?;
    let pr = &ctx.cfg.probes;
    let build = TestBuild::open(ctx, None)?;
    latches_release(&build.keyboard()?)?;
    let doc = match kind {
        AppKind::Explorer => probe::open_folder(ctx, &pr.ctrl_files, &[])?,
        _ => probe::open_text(ctx, kind, &pr.text)?,
    };
    say!("G18 {name}: {}", win::describe(doc.app.hwnd));
    let since = winclip::now();
    let run = |d: &Doc| clipkeep::keep(ctx.cfg, || drive(ctx, &uia, &build, d));
    let result = probe::run_on(ctx, doc, run);
    let off = build.set(RCLICK, false).err();
    let off = off.map(|e| format!("Right-click stayed on: {e}"));
    // Clean the history even when the run failed.
    let history = probe::forget_copies(ctx, since, &|t| probe::probe_copy(ctx, t));
    let closed = build.close();
    let (case, mut notes) = result.map_err(|e| {
        let off = off.as_deref().unwrap_or_default();
        format!(
            "{e}; {off}; history removed: {history}; test build: {}",
            closed.json()
        )
    })?;
    notes.extend(off);
    let pass = case["ok"] == json!(true);
    say!("G18 {name}: pass {pass}; {case}");
    Ok(json!({
        "gate": "G18", "app": name, "pass": pass, "case": case, "clean_up": notes, "history_removed": history,
        "test_build": closed.json(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_keyboard_whose_latches_stay_is_refused() {
        let mut s = KeyboardSettings::from_defaults().expect("the defaults load");
        s.modifiers = Modifiers::Release;
        assert_eq!(latches_release(&s), Ok(()));
        s.modifiers = Modifiers::Stay;
        let e = latches_release(&s).expect_err("a latch that stays");
        assert!(e.contains("release"), "{e}");
    }
}
