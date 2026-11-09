//! G19 core: a line selected with the keyboard is read back through UI Automation, never the clipboard; `keyxtend`,
//! `translate-read` and `translate-beside` check the selection strip (`g19chips`). The owner never uses Word, so core
//! runs in Notepad and Chrome alone and can pass in full.
#![cfg(windows)]

use kx_platform_windows::clip::open;
use kx_platform_windows::uia::{self, Sel, Uia};
use serde_json::{Value, json};
use spike_core::geom;

use crate::apps::{AppKind, Ctx, Opened};
use crate::ownkeys::{self, Real};
use crate::probe::{self, Doc};
use crate::win::{self, sleep_ms};
use crate::winclip::line;
use crate::{g19chips, launch, product, simuser};

/// G19's detection run, each line read back through UI Automation.
pub const CORE: &str = "core";
/// G19's run names: detection, the strip on the test build, the reads of the owner's translate page (on the test
/// build or beside the owner's own keyboard), the strip in the terminals alone, and the product's look.
pub const NAMES: [&str; 6] = [
    CORE,
    product::NAME,
    g19chips::READ,
    g19chips::BESIDE,
    g19chips::TERMINALS,
    g19chips::LOOK,
];
/// The apps G19 core selects in.
const APPS: [AppKind; 2] = [AppKind::Notepad, AppKind::Chrome];

/// Selects line `i` with Ctrl+Home, Down `i` times, Home and Shift+End, each only while our tab shows and each
/// followed by `key_settle_ms`, as Notepad can take a key late and lose the selection.
fn select_line(ctx: &Ctx, app: &Opened, i: usize) -> Result<(), String> {
    let ek = &ctx.cfg.edit_keys;
    let mut run: Vec<&[u16]> = vec![&ek.doc_start];
    run.extend(std::iter::repeat_n(ek.next_line.as_slice(), i));
    run.extend([ek.line_start.as_slice(), ek.select_line.as_slice()]);
    ownkeys::paced(&Real, app, &run, ctx.cfg.timing.key_settle_ms)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    Ok(())
}

/// The selection of the focused element in `app`'s program, once UI Automation shows its text and a box.
fn read_sel(ctx: &Ctx, uia: &Uia, app: &Opened) -> Result<Sel, String> {
    let (t, max, (pid, _)) = (&ctx.cfg.timing, ctx.cfg.g19.max_chars, win::owner(app.hwnd));
    win::poll_until(t.read_wait_ms, t.poll_ms, || {
        let f = uia
            .focused()
            .ok()
            .filter(|f| pid != 0 && uia::pid(f) == pid)?;
        uia::selection(&uia.text_of(&f)?, max)
            .ok()?
            .filter(|s| !s.boxes.is_empty())
    })
    .ok_or_else(|| why(uia, max, pid))
}

/// Which step of the selection read fails, for the error message.
fn why(uia: &Uia, max: usize, pid: u32) -> String {
    let f = match uia.focused() {
        Ok(f) => f,
        Err(e) => return e.to_string(),
    };
    let who = uia::kind(&f);
    if uia::pid(&f) != pid {
        return format!("focus on {who} in another program");
    }
    let Some(tp) = uia.text_of(&f) else {
        return format!("focus on {who}: no TextPattern on it or above");
    };
    match uia::selection(&tp, max) {
        Err(e) => format!("focus on {who}: {e}"),
        Ok(None) => format!("focus on {who}: the selection is empty"),
        Ok(Some(_)) => format!("focus on {who}: the selection has text but no box"),
    }
}

/// Selects and reads line `i`; its text is logged only when it is the expected test line.
fn check_line(ctx: &Ctx, uia: &Uia, app: &Opened, i: usize, want: &str) -> Value {
    let sel = match select_line(ctx, app, i).and_then(|()| read_sel(ctx, uia, app)) {
        Ok(s) => s,
        Err(e) => return json!({ "line": i, "ok": false, "error": e }),
    };
    let text = line(&sel.text);
    let ok = text == want;
    let shown = if ok {
        json!(text)
    } else {
        json!({ "chars": text.chars().count() })
    };
    json!({ "line": i, "ok": ok, "text": shown, "boxes": sel.boxes.len() })
}

/// Checks every line in `app`, and that the clipboard never changed.
fn check_all(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let uia = Uia::new().map_err(|e| e.to_string())?;
    let seq = open::sequence();
    let lines: Vec<Value> = (ctx.cfg.g19.lines.iter().enumerate())
        .map(|(i, want)| check_line(ctx, &uia, app, i, want))
        .collect();
    let untouched = open::sequence() == seq;
    let ok = untouched && lines.iter().all(|l| l["ok"] == true);
    Ok(
        json!({ "app": app.kind.name(), "ok": ok, "lines": lines, "clipboard_untouched": untouched }),
    )
}

/// Chrome on a page whose text box holds the lines; the box is clicked first so it has the focus.
fn open_chrome(ctx: &Ctx, text: &str) -> Result<Doc, String> {
    let g = &ctx.cfg.g19;
    let body = format!(
        "<textarea aria-label=\"{}\" wrap=\"off\" rows=\"{}\" style=\"{}\">{}</textarea>",
        g.box_name,
        g.lines.len(),
        g.box_css,
        launch::html_text(text)
    );
    probe::open_page(ctx, &body)
}

/// Clicks Chrome's text box so the keys go into it.
fn focus_box(ctx: &Ctx, app: &Opened) -> Result<(), String> {
    let (t, uia) = (&ctx.cfg.timing, Uia::new().map_err(|e| e.to_string())?);
    let el = win::poll_until(t.read_wait_ms, t.poll_ms, || {
        uia.named(app.hwnd, &ctx.cfg.g19.box_name)
    })
    .ok_or("no text box through UI Automation")?;
    simuser::click(
        ctx,
        app,
        geom::centre(&uia::rect(&el).map_err(|e| e.to_string())?),
    )
}

/// One app's run: open, check every line, close.
fn one(ctx: &Ctx, kind: AppKind) -> Value {
    let text = ctx.cfg.g19.lines.join("\n");
    let doc = match kind {
        AppKind::Chrome => open_chrome(ctx, &text),
        _ => probe::open_text(ctx, kind, &text),
    };
    let run = doc.and_then(|doc| {
        probe::run_on(ctx, doc, |doc| {
            probe::front(ctx, &doc.app)?;
            if kind == AppKind::Chrome {
                focus_box(ctx, &doc.app)?;
            }
            check_all(ctx, &doc.app)
        })
    });
    match run {
        Ok((mut v, notes)) => {
            v["clean_up"] = json!(notes);
            v
        }
        Err(e) => json!({ "app": kind.name(), "ok": false, "error": e }),
    }
}

/// Refuses any run name but G19's own; `facearg` refuses it first, so this guards a direct call.
fn only_ours(name: &str) -> Result<(), String> {
    if NAMES.contains(&name) {
        return Ok(());
    }
    Err(format!("G19 runs {}, not {name}", NAMES.join(", ")))
}

/// Runs G19: detection in Notepad, then Chrome, or the strip on the test build.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    only_ours(name)?;
    if name != CORE {
        return g19chips::run(ctx, name);
    }
    let apps: Vec<Value> = APPS.iter().map(|&k| one(ctx, k)).collect();
    let pass = apps.iter().all(|a| a["ok"] == true);
    Ok(json!({ "gate": "G19", "name": CORE, "pass": pass, "apps": apps }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g19_runs_its_own_names_only() {
        for name in NAMES {
            assert_eq!(only_ours(name), Ok(()), "{name}");
        }
        let e = only_ours(spike_core::config::FACE).expect_err("the face's pill is gone");
        assert!(NAMES.iter().all(|n| e.contains(n)), "{e}");
    }

    #[test]
    fn g19_core_never_waits_for_word() {
        assert_eq!(APPS, [AppKind::Notepad, AppKind::Chrome]);
        assert!(
            !APPS.contains(&AppKind::Word),
            "owner, 2026-10-16: never Word"
        );
    }
}
