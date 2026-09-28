//! G25: the language key cycles the layouts of the app in front, and the common shortcuts work.

use std::collections::HashSet;

use serde_json::{Value, json};
use spike_core::langkey;
use spike_core::layout::installed_layouts;
use spike_core::window::foreground;
use windows::Win32::Foundation::HWND;

use crate::apps::{AppKind, Ctx, Opened};
use crate::probe;
use crate::simuser::keys_ours;
use crate::win::{self, sleep_ms};
use crate::{clip, clipkeep, keys, readback, winclip};

/// The app G25 drives.
pub const APPS: [AppKind; 1] = [AppKind::Notepad];
/// Bits of a layout handle that hold its language id.
const LANG_MASK: isize = 0xFFFF;

/// A layout's language id in hex, such as `0409` for English (US).
fn lang(layout: isize) -> String {
    format!("{:04X}", layout & LANG_MASK)
}

/// Waits until `f` holds, up to G25's switch wait.
fn until(ctx: &Ctx, f: impl Fn() -> bool) -> bool {
    let (g, t) = (&ctx.cfg.g25, &ctx.cfg.timing);
    win::poll_until(g.switch_wait_ms, t.poll_ms, || f().then_some(())).is_some()
}

/// True once the app's focus shows `layout`.
fn shows(ctx: &Ctx, app: &Opened, layout: isize) -> bool {
    until(ctx, || langkey::layout_of(app.hwnd) == Some(layout))
}

/// Asks the app's focus for `layout`; true once it shows it within G25's switch wait.
pub(crate) fn switch_to(ctx: &Ctx, app: &Opened, layout: isize) -> bool {
    langkey::ask(app.hwnd, layout).is_ok() && shows(ctx, app, layout)
}

/// One press of the language key: the layout asked for and whether the app took it, or the error.
fn step(ctx: &Ctx, app: &Opened) -> Value {
    match keys_ours(app).and_then(|()| langkey::ask_next(app.hwnd)) {
        Ok(want) => json!({ "to": lang(want), "ok": shows(ctx, app, want) }),
        Err(e) => json!({ "ok": false, "error": e }),
    }
}

/// Asks for the next layout once per installed layout, so the list wraps; the first layout is always put back.
fn layouts(ctx: &Ctx, app: &Opened) -> Value {
    let Some(start) = langkey::layout_of(app.hwnd) else {
        return json!({ "error": "no layout for the app's focus", "ok": false });
    };
    let count = installed_layouts().len();
    if count < 2 {
        return json!({ "from": lang(start), "skipped": "needs two layouts", "ok": false });
    }
    let steps: Vec<Value> = (0..count).map(|_| step(ctx, app)).collect();
    let wrapped = langkey::layout_of(app.hwnd) == Some(start);
    let restored = wrapped || switch_to(ctx, app, start);
    let all = steps.iter().all(|s| s["ok"] == true);
    json!({
        "from": lang(start), "steps": steps, "wrapped": wrapped, "restored": restored,
        "ok": all && wrapped,
    })
}

/// Ctrl+A and Ctrl+C, Ctrl+End and Ctrl+V, then Ctrl+Z; each checked on the clipboard or the saved file.
fn shortcuts(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let (g, k, ek, t) = (
        &ctx.cfg.g25,
        &ctx.cfg.keys,
        &ctx.cfg.edit_keys,
        &ctx.cfg.timing,
    );
    keys_ours(app)?;
    keys::combo(&k.select_all)?;
    sleep_ms(t.key_settle_ms);
    let before = clip::sequence();
    keys_ours(app)?;
    keys::combo(&k.copy)?;
    let changed = clip::wait_change(before, t.read_wait_ms, t.poll_ms);
    let text = clip::read_text(t.read_wait_ms, t.poll_ms, ctx.cfg.g1.max_read_chars)?;
    let copy = changed && text.as_deref() == Some(g.text.as_str());
    keys_ours(app)?;
    keys::combo(&ek.doc_end)?;
    keys::combo(&ek.paste)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    let twice = g.text.repeat(2);
    let paste = readback::saved_until(ctx, app, |s| s.trim_end() == twice)?;
    keys_ours(app)?;
    keys::combo(&ek.undo)?;
    sleep_ms(ctx.cfg.probes.settle_ms);
    let undo = readback::saved_until(ctx, app, |s| s.trim_end() == g.text)?;
    Ok(json!({ "copy": copy, "paste": paste, "undo": undo, "ok": copy && paste && undo }))
}

/// A window's class and program file name; never its title, which may be private.
fn about(w: HWND) -> Value {
    json!({ "class": win::class(w), "program": win::program_name(w) })
}

/// Win+V shows the Windows clipboard panel, and Esc closes it; Windows ignores an injected Win+V without uiAccess.
fn clip_window(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let g = &ctx.cfg.g25;
    keys_ours(app)?;
    let before: HashSet<isize> = win::seen_windows().into_iter().map(win::key).collect();
    // The panel's windows exist all the time; they only stop being cloaked while it shows.
    let shown = || -> Vec<HWND> {
        let panel =
            |w: &HWND| !before.contains(&win::key(*w)) && g.panel_classes.contains(&win::class(*w));
        win::seen_windows().into_iter().filter(panel).collect()
    };
    keys::slow_combo(&g.win_v, ctx.cfg.timing.key_settle_ms)?;
    let opened = until(ctx, || !shown().is_empty());
    let new: Vec<Value> = shown().into_iter().map(about).collect();
    // Esc is sent only while our window or the panel is in front, so it never reaches another app.
    let safe = until(ctx, || {
        let f = foreground();
        f == app.hwnd || shown().contains(&f)
    });
    let at_esc = about(foreground());
    if safe {
        keys::combo(&g.esc)?;
    }
    let closed = safe && until(ctx, || foreground() == app.hwnd && shown().is_empty());
    let ok = opened && closed;
    Ok(json!({
        "opened": opened, "panel": new, "esc_sent": safe, "front_at_esc": at_esc,
        "closed": closed, "ok": ok,
    }))
}

/// Alt+Tab moves our window away, and Alt+Tab again brings it back; nothing is clicked.
fn alt_tab(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    let g = &ctx.cfg.g25;
    keys_ours(app)?;
    keys::combo(&g.alt_tab)?;
    let away = until(ctx, || foreground() != app.hwnd);
    let other = about(foreground());
    sleep_ms(ctx.cfg.timing.key_settle_ms);
    keys::combo(&g.alt_tab)?;
    let back = until(ctx, || foreground() == app.hwnd);
    if !back {
        let _ = probe::front(ctx, app);
    }
    Ok(json!({ "away": away, "other": other, "back": back, "ok": away && back }))
}

/// Every part in turn on Notepad.
fn drive(ctx: &Ctx, app: &Opened) -> Result<Value, String> {
    probe::front(ctx, app)?;
    let layouts = layouts(ctx, app);
    let shortcuts = clipkeep::keep(ctx.cfg, || shortcuts(ctx, app))?;
    let window = clip_window(ctx, app)?;
    probe::front(ctx, app)?;
    let tab = alt_tab(ctx, app)?;
    let pass = [&layouts, &shortcuts, &window, &tab]
        .iter()
        .all(|p| p["ok"] == true);
    Ok(json!({
        "gate": "G25", "app": app.kind.name(), "pass": pass,
        "layouts": layouts, "shortcuts": shortcuts, "clipboard_window": window, "alt_tab": tab,
    }))
}

/// Runs G25 on `name`; its copied text leaves Windows clipboard history even when the run fails.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kind = probe::kind_of(name, &APPS)?;
    let (g, since) = (&ctx.cfg.g25, winclip::now());
    let doc = probe::open_text(ctx, kind, &g.text)?;
    let result = probe::run_on(ctx, doc, |doc| drive(ctx, &doc.app));
    let history = probe::forget_copies(ctx, since, &|t| winclip::has_prefix(t, &[&g.text]));
    let (mut v, notes) = result.map_err(|e| format!("{e}; history removed: {history}"))?;
    v["history_removed"] = history;
    v["clean_up"] = json!(notes);
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lang_is_the_low_word_in_hex() {
        assert_eq!(lang(0x0409_0409), "0409");
        assert_eq!(lang(0xF002_1C01_u32 as isize), "1C01");
    }
}
