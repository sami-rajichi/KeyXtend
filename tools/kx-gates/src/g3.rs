//! G3 top band: opens Start, Search and Task Manager and checks the product keyboard is still on top of them.
#![cfg(windows)]

use std::collections::HashSet;
use std::ffi::c_void;

use kx_platform_windows::window::foreground;
use serde_json::{Value, json};
use spike_core::hold::Pt;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, POINT};
use windows::Win32::Security::{GetTokenInformation, TOKEN_QUERY, TokenUIAccess};
use windows::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::apps::Ctx;
use crate::gatecfg::Surface;
use crate::keys;
use crate::out::say;
use crate::product::Keyboard;
use crate::simin::inside;
use crate::win::{self, root_at, size32, sleep_ms};
use crate::winfind::{self, Match};

/// What probing one surface showed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// The surface overlapped at least one probe and the keyboard stayed on top at every probe.
    Pass,
    /// Something else was on top of the keyboard at a probe.
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

/// One probed key: its board id, the window on top there, and whether the surface covers it.
struct Hit {
    key: String,
    found: HWND,
    covered: bool,
}

/// The verdict for probes seen as (covered by the surface, keyboard on top).
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

/// True when every surface passed and the keyboard's token has uiAccess, as roadmap G3 is about the uiAccess build.
fn passed(surfaces: bool, ui_access: Option<bool>) -> bool {
    surfaces && ui_access == Some(true)
}

/// The uiAccess flag read from a token, or why it could not be read, as the result line keeps them.
pub(crate) fn token_fields(read: Result<bool, String>) -> (Option<bool>, Option<String>) {
    match read {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(e)),
    }
}

/// True if the process `pid` runs with the uiAccess flag in its token.
#[allow(unsafe_code, reason = "Both handles are ours and closed after use.")]
pub(crate) fn ui_access(pid: u32) -> Result<bool, String> {
    let mut value: u32 = 0;
    let mut len = 0u32;
    let mut token = HANDLE::default();
    // SAFETY: both handles are ours and closed before returning; `value` is a 4-byte buffer.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .map_err(|e| format!("OpenProcess {pid}: {e}"))?;
        let opened = OpenProcessToken(process, TOKEN_QUERY, &raw mut token);
        let _ = CloseHandle(process);
        opened.map_err(|e| format!("OpenProcessToken: {e}"))?;
        let buf = Some(std::ptr::from_mut(&mut value).cast::<c_void>());
        let read = GetTokenInformation(token, TokenUIAccess, buf, size32::<u32>(), &raw mut len);
        let _ = CloseHandle(token);
        read.map_err(|e| format!("GetTokenInformation: {e}"))?;
    }
    Ok(value != 0)
}

/// Closes what `surf` opened: its close keys, or `WM_CLOSE` to a new window of its class.
fn close(ctx: &Ctx, surf: &Surface, before: &HashSet<isize>) -> String {
    let tm = &ctx.cfg.timing;
    let mut notes = Vec::new();
    if !surf.close_keys.is_empty() {
        notes.push(
            keys::combo(&surf.close_keys).map_or_else(|e| e, |()| "close keys sent".to_string()),
        );
    }
    if let Some(class) = &surf.close_class {
        let want = Match {
            class: Some(class.clone()),
            skip: before.clone(),
            ..Default::default()
        };
        notes.push(match winfind::find(&want) {
            None => format!("no new {class} window; nothing closed"),
            Some(w) => match win::close(w) {
                Err(e) => format!("{e}; close it by hand"),
                Ok(()) if winfind::wait_gone(w, tm.close_wait_ms, tm.poll_ms) => {
                    "closed".to_string()
                }
                Ok(()) => "still open; close it by hand".to_string(),
            },
        });
    }
    sleep_ms(tm.key_settle_ms);
    notes.join("; ")
}

/// Opens `s`, then looks at what is on top of each probe and whether the surface covers it.
fn look(
    ctx: &Ctx,
    probes: &[(String, POINT)],
    s: &Surface,
) -> Result<(HWND, bool, Vec<Hit>), String> {
    let front0 = foreground();
    keys::combo(&s.open)?;
    sleep_ms(ctx.cfg.g3.open_wait_ms);
    let front = foreground();
    let opened = front != front0 && !front.is_invalid();
    let rect = win::rect(front).ok().filter(|_| opened);
    let hits = probes
        .iter()
        .map(|(key, p)| Hit {
            key: key.clone(),
            found: root_at(*p),
            covered: rect.is_some_and(|r| inside(&r, Pt { x: p.x, y: p.y })),
        })
        .collect();
    Ok((front, opened, hits))
}

/// Opens, probes and closes one surface, then prints and returns its verdict.
fn probe(ctx: &Ctx, kb: HWND, probes: &[(String, POINT)], s: &Surface) -> Result<Value, String> {
    let before = win::snapshot();
    let (front, opened, hits) = look(ctx, probes, s)?;
    let closed = close(ctx, s, &before);
    let seen: Vec<(bool, bool)> = hits.iter().map(|h| (h.covered, h.found == kb)).collect();
    let v = verdict(opened, &seen);
    let front = win::describe(front);
    say!("{}: {}; front was {front}; {closed}", s.name, v.text());
    for h in hits.iter().filter(|h| h.found != kb || !h.covered) {
        let (key, covered) = (&h.key, h.covered);
        say!(
            "  key {key}: found {}; covered {covered}",
            win::describe(h.found)
        );
    }
    let hits: Vec<Value> = hits
        .iter()
        .map(|h| {
            json!({ "key": h.key, "ok": h.found == kb,
            "covered": h.covered, "found": win::describe(h.found) })
        })
        .collect();
    Ok(json!({
        "surface": s.name, "verdict": v.text(), "pass": v == Verdict::Pass, "opened": opened,
        "front": front, "hits": hits, "closed": closed,
    }))
}

/// Runs G3 on the product keyboard, called `name`.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let kb = Keyboard::find()?;
    let probes = ctx
        .cfg
        .g3
        .probe_keys
        .iter()
        .map(|key| Ok((key.clone(), kb.at(key)?)));
    let probes = probes.collect::<Result<Vec<_>, String>>()?;
    let token = ui_access(win::owner(kb.hwnd).0);
    say!(
        "G3 {name}: keyboard {}; uiAccess in token: {token:?}",
        win::describe(kb.hwnd)
    );
    let surfaces = ctx
        .cfg
        .g3
        .surfaces
        .iter()
        .map(|s| probe(ctx, kb.hwnd, &probes, s))
        .collect::<Result<Vec<_>, _>>()?;
    let (ui, ui_error) = token_fields(token);
    let pass = passed(surfaces.iter().all(|s| s["pass"] == true), ui);
    Ok(json!({
        "gate": "G3", "keyboard": name, "pass": pass,
        "ui_access": ui, "ui_access_error": ui_error, "surfaces": surfaces,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_keyboard_on_top_where_the_surface_overlaps_passes() {
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
    fn g3_passes_only_for_a_keyboard_whose_token_has_ui_access() {
        assert!(passed(true, Some(true)));
        assert!(!passed(true, Some(false)), "a build without uiAccess");
        assert!(!passed(true, None), "a token that could not be read");
        assert!(!passed(false, Some(true)));
    }

    #[test]
    fn a_token_reads_as_its_flag_or_as_the_error() {
        assert_eq!(token_fields(Ok(true)), (Some(true), None));
        assert_eq!(
            token_fields(Err("OpenProcess 7: denied".into())),
            (None, Some("OpenProcess 7: denied".into()))
        );
    }
}
