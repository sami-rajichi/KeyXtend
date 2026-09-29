//! G19 core: a line selected with the keyboard is read back through UI Automation, never the clipboard.
#![cfg(windows)]

use serde_json::{Value, json};
use spike_core::screen::{self, work_area};
use spike_core::selwatch::PillPx;
use spike_core::uia::{self, Sel, Uia};

use crate::apps::{AppKind, Ctx, Opened};
use crate::probe::{self, Doc};
use crate::win::{self, sleep_ms};
use crate::winclip::line;
use crate::{clip, keys, launch, simuser};

/// G19's run name: detection only, before the pill exists.
pub const CORE: &str = "core";
/// The apps G19 selects in.
const APPS: [AppKind; 2] = [AppKind::Notepad, AppKind::Chrome];

/// Selects line `i` with Ctrl+Home, Down `i` times, Home and Shift+End.
fn select_line(ctx: &Ctx, app: &Opened, i: usize) -> Result<(), String> {
    let ek = &ctx.cfg.edit_keys;
    simuser::keys_ours(app)?;
    keys::combo(&ek.doc_start)?;
    for _ in 0..i {
        keys::combo(&ek.next_line)?;
    }
    keys::combo(&ek.line_start)?;
    keys::combo(&ek.select_line)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    Ok(())
}

/// The selection of the focused element in `app`'s program, once UI Automation shows its text and a box for the pill.
pub(crate) fn read_sel(ctx: &Ctx, uia: &Uia, app: &Opened) -> Result<Sel, String> {
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
        Err(e) => return e,
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
    let first = sel.boxes.first().copied().unwrap_or_default();
    let pill = PillPx(ctx.spike.tools.pill_px).at(screen::dpi_of(&first));
    let pill = work_area(&first)
        .ok()
        .and_then(|screen| uia::anchor(&sel.boxes, &screen, pill));
    let ok = text == want && pill.is_some();
    let shown = if text == want {
        json!(text)
    } else {
        json!({ "chars": text.chars().count() })
    };
    json!({ "line": i, "ok": ok, "text": shown, "boxes": sel.boxes.len(), "pill": pill.map(|p| [p.x, p.y]) })
}

/// Checks every line in `app`, and that the clipboard never changed.
fn check_all(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let uia = Uia::new()?;
    let seq = clip::sequence();
    let lines: Vec<Value> = (ctx.cfg.g19.lines.iter().enumerate())
        .map(|(i, want)| check_line(ctx, &uia, app, i, want))
        .collect();
    let untouched = clip::sequence() == seq;
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
    let (t, uia) = (&ctx.cfg.timing, Uia::new()?);
    let el = win::poll_until(t.read_wait_ms, t.poll_ms, || {
        uia.named(app.hwnd, &ctx.cfg.g19.box_name)
    })
    .ok_or("no text box through UI Automation")?;
    simuser::click(ctx, app, uia::centre(&uia::rect(&el)?))
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

/// Runs G19 detection in Notepad, then Chrome; any other name is a face whose pill is checked.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name != CORE {
        return crate::g19pill::run(ctx, name);
    }
    let apps: Vec<Value> = APPS.iter().map(|&k| one(ctx, k)).collect();
    let pass = apps.iter().all(|a| a["ok"] == true);
    Ok(json!({ "gate": "G19", "name": CORE, "pass": pass, "apps": apps }))
}
