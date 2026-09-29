//! G19 pill: a face's pill shows next to a keyboard selection without taking focus, copies it, and hides after a click elsewhere.
#![cfg(windows)]

use std::time::Instant;

use serde_json::{Value, json};
use spike_core::hold::Pt;
use spike_core::screen::{self, work_area};
use spike_core::selwatch::{self, PillPx};
use spike_core::uia::{self, Uia};
use spike_core::window::foreground;
use windows::Win32::Foundation::{HWND, RECT};

use crate::apps::{AppKind, Ctx, Opened};
use crate::facetools::{self, FaceWin};
use crate::win::{self, sleep_ms};
use crate::winclip::{self, line};
use crate::{clip, clipkeep, cliptext, g19, probe, simuser};

/// Where kx-gates expects the pill for the current selection, from its own UI Automation read.
fn expected(ctx: &Ctx, uia: &Uia, app: &Opened) -> Result<Pt, String> {
    let sel = g19::read_sel(ctx, uia, app)?;
    let first = *sel.boxes.first().ok_or("the selection has no box")?;
    let screen = work_area(&first)?;
    let pill = PillPx(ctx.spike.tools.pill_px).at(screen::dpi_of(&first));
    selwatch::place(&sel.boxes, &screen, pill).ok_or_else(|| "no pill place".into())
}

/// Waits until the pill shows (`show`) or hides; returns the ms since `t0`, the step that should cause it.
fn wait_pill(ctx: &Ctx, show: bool, t0: Instant) -> Option<u128> {
    let title = &ctx.spike.tools.pill_title;
    let spent = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
    let left = ctx.cfg.g19.pill_wait_ms.saturating_sub(spent);
    let done = || (facetools::shown(title).is_some() == show).then(|| t0.elapsed().as_millis());
    win::poll_until(left, ctx.cfg.timing.poll_ms, done)
}

/// Line breaks at the end of `text`, which a line range holds but a Shift+End selection does not.
fn trailing_breaks(text: &str) -> i32 {
    let n = text.len() - text.trim_end_matches(['\r', '\n']).len();
    i32::try_from(n).unwrap_or_default()
}

/// Selects line `i` through UI Automation: no input reaches the app, so the face keeps the last input and could take focus.
#[allow(unsafe_code, reason = "Plain COM calls on live UI Automation objects.")]
fn select_quietly(uia: &Uia, app: &Opened, i: usize) -> Result<(), String> {
    use windows::Win32::UI::Accessibility::{
        TextPatternRangeEndpoint_End as END, TextPatternRangeEndpoint_Start as START,
        TextUnit_Character, TextUnit_Line,
    };
    let doc = uia
        .text(app.hwnd)
        .ok_or("no text document through UI Automation")?;
    let tp = uia.text_of(&doc).ok_or("no text pattern")?;
    let lines = i32::try_from(i).map_err(|e| e.to_string())?;
    let fail = |e: windows::core::Error| format!("UIA select: {e}");
    // SAFETY: plain COM calls on live UI Automation objects.
    unsafe {
        let r = tp.DocumentRange().map_err(fail)?;
        r.MoveEndpointByRange(END, &r, START).map_err(fail)?;
        r.Move(TextUnit_Line, lines).map_err(fail)?;
        r.ExpandToEnclosingUnit(TextUnit_Line).map_err(fail)?;
        let text = r.GetText(-1).map_err(fail)?.to_string();
        let back = -trailing_breaks(&text);
        r.MoveEndpointByUnit(END, TextUnit_Character, back)
            .map_err(fail)?;
        r.Select().map_err(fail)
    }
}

/// Clicks the pill's Copy and reads the clipboard; true when it holds the selected line.
fn copy_via(ctx: &Ctx, pill: HWND, at: &RECT, want: &str) -> Result<bool, String> {
    let t = &ctx.cfg.timing;
    let before = clip::sequence();
    facetools::click_on(pill, facetools::at(uia::centre(at)))?;
    if !clip::wait_change(before, t.read_wait_ms, t.poll_ms) {
        return Ok(false);
    }
    let got = cliptext::read_text(t.read_wait_ms, t.poll_ms, ctx.cfg.g19.max_chars)?;
    Ok(got.as_deref().map(line) == Some(want))
}

/// Clicks Notepad's text below the lines, which drops the selection.
fn click_elsewhere(ctx: &Ctx, uia: &Uia, app: &Opened) -> Result<(), String> {
    let doc = uia
        .text(app.hwnd)
        .ok_or("no text document through UI Automation")?;
    let r = uia::rect(&doc)?;
    let inset = screen::physical(ctx.spike.keyboard.key_px, screen::dpi_of(&r));
    let p = Pt {
        x: r.left.saturating_add(inset),
        y: r.bottom.saturating_sub(inset),
    };
    simuser::click(ctx, app, p)
}

/// One line: click the face as a user would, select the line quietly, then check the pill's place, focus, Copy and hiding.
fn one(ctx: &Ctx, uia: &Uia, fw: &FaceWin, app: &Opened, i: usize) -> Result<Value, String> {
    let (want, slack) = (&ctx.cfg.g19.lines[i], ctx.cfg.g19.pill_slack_px);
    facetools::click_on(fw.hwnd, fw.quiet_spot(ctx))?;
    let t0 = Instant::now();
    select_quietly(uia, app, i)?;
    let expect = expected(ctx, uia, app)?;
    let shown_ms = wait_pill(ctx, true, t0);
    let pill = facetools::shown(&ctx.spike.tools.pill_title);
    let at = pill.and_then(|p| win::rect(p).ok());
    let off = at.map(|r| [r.left - expect.x, r.top - expect.y]);
    let near = off.is_some_and(|[dx, dy]| dx.abs() <= slack && dy.abs() <= slack);
    let front_kept = foreground() == app.hwnd;
    let copied = match (pill, at) {
        (Some(p), Some(r)) => copy_via(ctx, p, &r, want)?,
        _ => false,
    };
    let front_after_copy = foreground() == app.hwnd;
    sleep_ms(ctx.cfg.timing.key_settle_ms);
    let t1 = Instant::now();
    click_elsewhere(ctx, uia, app)?;
    let hidden_ms = wait_pill(ctx, false, t1);
    let ok = shown_ms.is_some()
        && near
        && front_kept
        && copied
        && front_after_copy
        && hidden_ms.is_some();
    Ok(json!({
        "line": i, "ok": ok, "shown_ms": shown_ms, "offset_px": off, "front_kept": front_kept,
        "copied": copied, "front_after_copy": front_after_copy, "hidden_ms": hidden_ms,
    }))
}

/// Every line in turn; a line that fails early is reported and the next one still runs.
fn all(ctx: &Ctx, fw: &FaceWin, app: &Opened) -> Result<Vec<Value>, String> {
    let uia = Uia::new()?;
    probe::front(ctx, app)?;
    let n = ctx.cfg.g19.lines.len();
    Ok((0..n)
        .map(|i| {
            one(ctx, &uia, fw, app, i)
                .unwrap_or_else(|e| json!({ "line": i, "ok": false, "error": e }))
        })
        .collect())
}

/// Runs the pill check of face `name` in Notepad; the clipboard and Win+V history are cleaned even on failure.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let fw = FaceWin::parked(ctx, name)?;
    let (g, since) = (&ctx.cfg.g19, winclip::now());
    let doc = probe::open_text(ctx, AppKind::Notepad, &g.lines.join("\n"))?;
    let result = probe::run_on(ctx, doc, |doc| {
        clipkeep::keep(ctx.cfg, || all(ctx, &fw, &doc.app))
    });
    let history = probe::forget_copies(ctx, since, &|t| g.lines.iter().any(|l| l == line(t)));
    let (lines, notes) = result.map_err(|e| format!("{e}; history removed: {history}"))?;
    let pass = lines.iter().all(|l| l["ok"] == true);
    Ok(json!({
        "gate": "G19", "face": name, "pass": pass, "lines": lines,
        "history_removed": history, "clean_up": notes,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_line_breaks_at_the_end_are_trimmed() {
        assert_eq!(trailing_breaks("abc\r\n"), 2);
        assert_eq!(trailing_breaks("abc\r"), 1);
        assert_eq!(trailing_breaks("a\rbc"), 0);
        assert_eq!(trailing_breaks(""), 0);
    }
}
