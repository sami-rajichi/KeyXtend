//! G4 admin windows: face keys clicked into Windows Terminal running as administrator.
#![cfg(windows)]

use serde_json::{Value, json};
use windows::Win32::Foundation::HWND;

use crate::apps::{Ctx, Opened};
use crate::clicks::{self, Tally};
use crate::diff::{self, Diff};
use crate::mouse::{self, Face};
use crate::win::{self, sleep_ms};
use crate::{g1, launch, readback};

/// What the clicks did, and the line the admin shell saved.
struct Typed {
    tally: Tally,
    /// Keys left out because another window covered them.
    hidden: Vec<String>,
    /// The saved line, or why it could not be read.
    got: Result<String, String>,
}

/// True when the shell ran as administrator, every click kept its focus, and the text arrived exactly.
fn passed(admin: bool, tally: &Tally, equal: bool) -> bool {
    let n = tally.clicks.len();
    admin && equal && n > 0 && tally.focus_kept == n && tally.stopped.is_none()
}

/// Waits for the shell's text box, clicks face keys, then presses Enter to save the line.
fn type_into(
    ctx: &Ctx,
    term: &Opened,
    face: (&Face, HWND),
    run: (usize, u64),
) -> Result<Typed, String> {
    let app = ctx.cfg.app(term.kind.name())?;
    g1::wait_ready(ctx, term, app.focus_class.as_deref())?;
    // Keys are read once the shell is in front, so they follow its keyboard layout.
    let (keys, hidden) = clicks::usable(ctx, face.0, face.1)?;
    let tally = clicks::click_loop(ctx, term, face.1, &keys, run);
    sleep_ms(app.done_ms);
    let got = match &tally.stopped {
        Some(_) => Err("skipped: clicking stopped".to_string()),
        None => readback::read_back(ctx, term),
    };
    Ok(Typed { tally, hidden, got })
}

/// Closes the admin shell if saving the line did not end it; returns what is left open.
fn close(ctx: &Ctx, term: &Opened) -> Vec<String> {
    let t = &ctx.cfg.timing;
    if win::wait_gone(term.hwnd, t.close_wait_ms, t.poll_ms) {
        return Vec::new();
    }
    let _ = win::close(term.hwnd);
    if win::wait_gone(term.hwnd, t.close_wait_ms, t.poll_ms) {
        Vec::new()
    } else {
        vec![format!("left open: {}", win::describe(term.hwnd))]
    }
}

/// Runs G4 on the face called `name` with `count` clicks from `seed`.
pub fn run(ctx: &Ctx, name: &str, count: usize, seed: u64) -> Result<Value, String> {
    let hwnd = mouse::find_face(ctx.spike, name)?;
    let face = mouse::face(hwnd)?;
    let face_program = win::program(hwnd).unwrap_or_else(|e| e);
    let (term, admin) = launch::open_admin(ctx)?;
    let window = win::describe(term.hwnd);
    println!(
        "G4 {name}: {count} clicks, seed {seed}; {window}; admin {admin}; face {face_program}"
    );
    let typed = type_into(ctx, &term, (&face, hwnd), (count, seed));
    let left_open = close(ctx, &term);
    let t = typed?;
    let (got, read_error) = match &t.got {
        Ok(s) => (s.as_str(), None),
        Err(e) => ("", Some(e.clone())),
    };
    let d = diff::compare(
        &clicks::typed(&t.tally.clicks),
        got,
        ctx.cfg.g1.context_chars,
    );
    let pass = passed(admin, &t.tally, d.equal);
    print(admin, &t.tally, &d, pass);
    if let Some(e) = &read_error {
        println!("read back failed: {e}");
    }
    Ok(json!({
        "gate": "G4", "face": name, "face_program": face_program, "seed": seed, "planned": count,
        "window": window, "admin": admin, "pass": pass,
        "clicks": t.tally.clicks.len(), "focus_kept": t.tally.focus_kept,
        "failures": t.tally.failures, "stopped": t.tally.stopped,
        "covered_skips": t.tally.covered, "covered_by": t.tally.covered_by,
        "hidden_keys": t.hidden, "read_error": read_error, "diff": d, "left_open": left_open,
    }))
}

/// Prints the verdict and what decided it.
fn print(admin: bool, tally: &Tally, d: &Diff, pass: bool) {
    let yes = |b: bool| if b { "yes" } else { "no" };
    println!(
        "admin {}; clicks {}; focus kept {}; chars sent {}, received {}; equal {}; pass {}",
        yes(admin),
        tally.clicks.len(),
        tally.focus_kept,
        d.sent_len,
        d.received_len,
        yes(d.equal),
        yes(pass)
    );
    if let Some(s) = &tally.stopped {
        println!("stopped: {s}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Click;

    fn tally(clicks: usize, focus_kept: usize, stopped: Option<&str>) -> Tally {
        let click = Click {
            code: 0,
            us: 0,
            expect: vec![],
        };
        Tally {
            clicks: vec![click; clicks],
            focus_kept,
            stopped: stopped.map(String::from),
            ..Default::default()
        }
    }

    #[test]
    fn passes_only_as_admin_with_all_focus_kept_and_equal_text() {
        assert!(passed(true, &tally(3, 3, None), true));
        assert!(!passed(false, &tally(3, 3, None), true));
        assert!(!passed(true, &tally(3, 2, None), true));
        assert!(!passed(true, &tally(3, 3, None), false));
        assert!(!passed(true, &tally(3, 3, Some("stuck")), true));
        assert!(!passed(true, &tally(0, 0, None), true));
    }
}
