//! G3 top band: opens Start, Search and Task Manager and checks the face is still on top of them.
#![cfg(windows)]

use std::collections::HashSet;
use std::ffi::c_void;

use serde_json::{Value, json};
use spike_core::window::foreground;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, POINT, RECT};
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUIAccess};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

use crate::apps::Ctx;
use crate::config::Surface;
use crate::win::{self, Match, root_at, sleep_ms};
use crate::{keys, mouse};

/// What probing one surface showed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// The surface overlapped at least one probe and the face stayed on top at every probe.
    Pass,
    /// Something else was on top of the face at a probe.
    Fail,
    /// The surface did not come to the front, so nothing was tested.
    NotOpened,
    /// The surface reached no probe, so nothing was tested.
    NotCovered,
}

impl Verdict {
    /// The word printed and saved for this verdict.
    fn text(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::NotOpened => "not opened",
            Self::NotCovered => "not covered",
        }
    }
}

/// One probed key: its scan code, the window on top there, and whether the surface covers it.
struct Hit {
    code: u32,
    found: HWND,
    covered: bool,
}

/// The verdict for probes seen as (covered by the surface, face on top).
fn verdict(opened: bool, probes: &[(bool, bool)]) -> Verdict {
    if probes.iter().any(|&(_, on_top)| !on_top) {
        Verdict::Fail
    } else if !opened {
        Verdict::NotOpened
    } else if !probes.iter().any(|&(covered, _)| covered) {
        Verdict::NotCovered
    } else {
        Verdict::Pass
    }
}

/// True if `p` is inside `r`; the right and bottom edges are outside.
fn contains(r: RECT, p: POINT) -> bool {
    (r.left..r.right).contains(&p.x) && (r.top..r.bottom).contains(&p.y)
}

/// The screen box of `hwnd`, when Windows gives one.
#[allow(unsafe_code, reason = "Plain query into a local.")]
fn window_rect(hwnd: HWND) -> Option<RECT> {
    let mut r = RECT::default();
    // SAFETY: plain query into a local; a bad handle gives an error.
    unsafe { GetWindowRect(hwnd, &mut r) }.ok().map(|()| r)
}

/// True if the process `pid` runs with the uiAccess flag in its token.
#[allow(unsafe_code, reason = "Both handles are ours and closed after use.")]
fn ui_access(pid: u32) -> Result<bool, String> {
    let mut value: u32 = 0;
    let mut len = 0u32;
    let mut token = HANDLE::default();
    // SAFETY: both handles are ours and closed before returning; `value` is a 4-byte buffer.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess {pid}: {e}"))?;
        let opened = OpenProcessToken(process, TOKEN_QUERY, &mut token);
        let _ = CloseHandle(process);
        opened.map_err(|e| format!("OpenProcessToken: {e}"))?;
        let buf = Some(std::ptr::from_mut(&mut value).cast::<c_void>());
        let read =
            GetTokenInformation(token, TokenUIAccess, buf, size_of::<u32>() as u32, &mut len);
        let _ = CloseHandle(token);
        read.map_err(|e| format!("GetTokenInformation: {e}"))?;
    }
    Ok(value != 0)
}

/// Closes what `s` opened: its close keys, or `WM_CLOSE` to a new window of its class.
fn close(ctx: &Ctx, s: &Surface, before: &HashSet<isize>) -> String {
    let t = &ctx.cfg.timing;
    let mut notes = Vec::new();
    if !s.close_keys.is_empty() {
        notes.push(
            keys::combo(&s.close_keys).map_or_else(|e| e, |()| "close keys sent".to_string()),
        );
    }
    if let Some(class) = &s.close_class {
        let m = Match {
            class: Some(class.clone()),
            skip: before.clone(),
            ..Default::default()
        };
        notes.push(match win::find(&m) {
            None => format!("no new {class} window; nothing closed"),
            Some(w) => match win::close(w) {
                Err(e) => format!("{e}; close it by hand"),
                Ok(()) if win::wait_gone(w, t.close_wait_ms, t.poll_ms) => "closed".to_string(),
                Ok(()) => "still open; close it by hand".to_string(),
            },
        });
    }
    sleep_ms(t.key_settle_ms);
    notes.join("; ")
}

/// Opens `s`, then looks at what is on top of each probe and whether the surface covers it.
fn look(ctx: &Ctx, probes: &[(u32, POINT)], s: &Surface) -> Result<(HWND, bool, Vec<Hit>), String> {
    let front0 = foreground();
    keys::combo(&s.open)?;
    sleep_ms(ctx.cfg.g3.open_wait_ms);
    let front = foreground();
    let opened = front != front0 && !front.is_invalid();
    let rect = window_rect(front).filter(|_| opened);
    let hits = probes
        .iter()
        .map(|&(code, p)| Hit {
            code,
            found: root_at(p),
            covered: rect.is_some_and(|r| contains(r, p)),
        })
        .collect();
    Ok((front, opened, hits))
}

/// Opens, probes and closes one surface, then prints and returns its verdict.
fn probe(ctx: &Ctx, face: HWND, probes: &[(u32, POINT)], s: &Surface) -> Result<Value, String> {
    let before = win::snapshot();
    let (front, opened, hits) = look(ctx, probes, s)?;
    let closed = close(ctx, s, &before);
    let seen: Vec<(bool, bool)> = hits.iter().map(|h| (h.covered, h.found == face)).collect();
    let v = verdict(opened, &seen);
    let front = win::describe(front);
    println!("{}: {}; front was {front}; {closed}", s.name, v.text());
    for h in hits.iter().filter(|h| h.found != face || !h.covered) {
        let (code, covered) = (h.code, h.covered);
        println!(
            "  key {code:#X}: found {}; covered {covered}",
            win::describe(h.found)
        );
    }
    let hits: Vec<Value> = hits
        .iter()
        .map(|h| {
            json!({ "key": format!("{:#X}", h.code), "ok": h.found == face,
            "covered": h.covered, "found": win::describe(h.found) })
        })
        .collect();
    Ok(json!({
        "surface": s.name, "verdict": v.text(), "pass": v == Verdict::Pass, "opened": opened,
        "front": front, "hits": hits, "closed": closed,
    }))
}

/// Runs G3 on the face called `name`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let face_hwnd = mouse::find_face(ctx.spike, name)?;
    let face = mouse::face(face_hwnd)?;
    let places = mouse::key_places(&ctx.spike.keyboard);
    let mut probes = Vec::new();
    for &code in &ctx.cfg.g3.probe_codes {
        let (_, place) = places
            .iter()
            .find(|(c, _)| *c == code)
            .ok_or_else(|| format!("probe key {code:#X} is not in spike.toml"))?;
        probes.push((code, mouse::centre(place, face.origin, face.dpi)));
    }
    let token = ui_access(win::owner(face_hwnd).0);
    println!(
        "G3 {name}: face {}; uiAccess in token: {token:?}",
        win::describe(face_hwnd)
    );
    let surfaces = ctx
        .cfg
        .g3
        .surfaces
        .iter()
        .map(|s| probe(ctx, face_hwnd, &probes, s))
        .collect::<Result<Vec<_>, _>>()?;
    let pass = surfaces.iter().all(|s| s["pass"] == true);
    let (ui, ui_error) = match token {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(e)),
    };
    Ok(json!({
        "gate": "G3", "face": name, "pass": pass,
        "ui_access": ui, "ui_access_error": ui_error, "surfaces": surfaces,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_face_on_top_where_the_surface_overlaps_passes() {
        let all = [(true, true), (true, true)];
        assert_eq!(verdict(true, &all), Verdict::Pass);
        assert_eq!(verdict(false, &all), Verdict::NotOpened);
        assert_eq!(
            verdict(true, &[(true, true), (false, true)]),
            Verdict::Pass,
            "a probe the surface does not reach tests nothing"
        );
        assert_eq!(
            verdict(true, &[(false, true), (false, true)]),
            Verdict::NotCovered
        );
        assert_eq!(
            verdict(true, &[(true, false), (false, true)]),
            Verdict::Fail
        );
        assert_eq!(verdict(false, &[(false, false)]), Verdict::Fail);
        assert_eq!(verdict(true, &[]), Verdict::NotCovered);
    }

    #[test]
    fn contains_keeps_the_right_and_bottom_edges_out() {
        let r = RECT {
            left: 0,
            top: 10,
            right: 100,
            bottom: 50,
        };
        assert!(contains(r, POINT { x: 0, y: 10 }));
        assert!(contains(r, POINT { x: 99, y: 49 }));
        assert!(!contains(r, POINT { x: 100, y: 20 }));
        assert!(!contains(r, POINT { x: 20, y: 50 }));
        assert!(!contains(r, POINT { x: -1, y: 20 }));
    }
}
