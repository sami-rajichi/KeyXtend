//! G1 typing: random EN/FR/AR/AltGr text typed into one app, read back and compared exactly.

use std::path::PathBuf;

use serde_json::{Value, json};
use spike_core::window::foreground;
use windows::Win32::Foundation::HWND;

use crate::apps::{self, AppKind, Ctx, Opened};
use crate::diff::{self, Diff};
use crate::rng::Rng;
use crate::win::{self, sleep_ms};
use crate::{launch, out, readback, shot, text};

/// Line endings some apps add to copied text; removed before comparing and counted.
const LINE_ENDS: [char; 2] = ['\r', '\n'];
/// Suffix of the file with the text we sent.
const SENT_FILE: &str = "-sent.txt";
/// Suffix of the file with the text we read back.
const RECEIVED_FILE: &str = "-received.txt";
/// Suffix of the screenshot taken at the first problem.
const SCREEN_FILE: &str = "-screen.bmp";

/// One G1 run: how many characters, the random seed, and how to type them.
pub struct Run {
    /// Characters to type.
    pub count: usize,
    /// Seed of the random text.
    pub seed: u64,
    /// Use the app's open window instead of starting it.
    pub attach: bool,
    /// Pause after each chunk of characters, in ms.
    pub pause_ms: u64,
}

/// What typing did, and what held the keyboard.
struct Typing {
    typed: usize,
    stopped: Option<String>,
    /// The control that held the keyboard when typing ended.
    focus: String,
    /// Pop-ups and a screenshot, taken at the first problem.
    evidence: Value,
}

/// True when the focus is `inside` the app window and has the wanted class, if one is wanted.
fn focus_ok(inside: bool, class: &str, want: Option<&str>) -> bool {
    inside && want.is_none_or(|w| w.eq_ignore_ascii_case(class))
}

/// True when the window title shows the test file, if the app has one.
fn title_ok(title: &str, has: Option<&str>) -> bool {
    has.is_none_or(|h| title.contains(h))
}

/// `Err` names what holds the keyboard when the app, its test file or its text box does not.
fn ready(app: &Opened, want: Option<&str>) -> Result<(), String> {
    let (hwnd, front) = (app.hwnd, foreground());
    if front != hwnd {
        return Err(format!("{} is in front", win::describe(front)));
    }
    if !title_ok(&win::title(hwnd), app.title_has.as_deref()) {
        return Err(format!(
            "the test file left the front: {}",
            win::describe(hwnd)
        ));
    }
    let focus = win::focus(hwnd);
    if focus_ok(win::contains(hwnd, focus), &win::class(focus), want) {
        Ok(())
    } else {
        Err(format!("keyboard focus is on {}", win::describe(focus)))
    }
}

/// Brings the app to the front, then waits for its text box to take the keyboard.
fn wait_ready(ctx: &Ctx, app: &Opened, want: Option<&str>) -> Result<(), String> {
    let t = &ctx.cfg.timing;
    if !win::front(app.hwnd, t, &ctx.cfg.keys) {
        return Err(format!(
            "could not bring it to the front; {} is in front",
            win::describe(foreground())
        ));
    }
    let mut last = String::new();
    let state = || match ready(app, want) {
        Ok(()) => Some(()),
        Err(e) => {
            last = e;
            None
        }
    };
    win::poll_until(t.focus_wait_ms, t.poll_ms, state).ok_or(last)
}

/// Types in chunks, checking before each chunk that the text box still holds the keyboard.
fn type_text(
    ctx: &Ctx,
    app: &Opened,
    want: Option<&str>,
    run: &Run,
    text: &str,
) -> (usize, Option<String>) {
    let chars: Vec<char> = text.chars().collect();
    let mut typed = 0;
    for chunk in chars.chunks(ctx.cfg.g1.chunk_chars.max(1)) {
        if let Err(e) = ready(app, want) {
            return (typed, Some(format!("blocked: {e}")));
        }
        if let Err(e) = spike_core::inject::text(&chunk.iter().collect::<String>()) {
            return (typed, Some(e));
        }
        typed += chunk.len();
        sleep_ms(run.pause_ms);
    }
    (typed, None)
}

/// The app's pop-ups, what is in front, and a screenshot, for a blocked run.
fn evidence(ctx: &Ctx, hwnd: HWND) -> Value {
    let file = ctx.file(SCREEN_FILE);
    let screenshot = match shot::save(&file) {
        Ok(()) => json!(file),
        Err(e) => json!({ "error": e }),
    };
    json!({
        "front": win::describe(foreground()), "focus": win::describe(win::focus(hwnd)),
        "popups": win::popups(hwnd), "screenshot": screenshot,
    })
}

/// Waits for the text box, types, and gathers evidence if typing stopped.
fn typing(ctx: &Ctx, app: &Opened, want: Option<&str>, run: &Run, sent: &str) -> Typing {
    let (typed, stopped) = match wait_ready(ctx, app, want) {
        Ok(()) => type_text(ctx, app, want, run, sent),
        Err(e) => (0, Some(format!("blocked: {e}"))),
    };
    let focus = win::describe(win::focus(app.hwnd));
    let evidence = match stopped {
        Some(_) => evidence(ctx, app.hwnd),
        None => Value::Null,
    };
    Typing {
        typed,
        stopped,
        focus,
        evidence,
    }
}

/// Reads the text back, unless typing stopped; adds evidence if reading fails.
fn read(ctx: &Ctx, app: &Opened, t: &mut Typing) -> (String, Option<String>) {
    let read = match &t.stopped {
        Some(_) if app.kind != AppKind::Target => Err("skipped: typing stopped".to_string()),
        _ => readback::read_back(ctx, app),
    };
    match read {
        Ok(text) => (text, None),
        Err(e) => {
            if t.evidence.is_null() {
                t.evidence = evidence(ctx, app.hwnd);
            }
            (String::new(), Some(e))
        }
    }
}

/// Makes the random text and saves it.
fn prepare(ctx: &Ctx, kind: AppKind, run: &Run) -> Result<(String, PathBuf), String> {
    let sent = text::generate(&ctx.cfg.text, run.count, &mut Rng::new(run.seed))?;
    let file = ctx.file(SENT_FILE);
    out::write(&file, &sent)?;
    println!(
        "G1 {}: {} chars, seed {}, pause {} ms",
        kind.name(),
        run.count,
        run.seed,
        run.pause_ms
    );
    Ok((sent, file))
}

/// Runs G1 on `kind`; the app is cleaned up before anything else can fail.
pub fn run(ctx: &Ctx, kind: AppKind, run: &Run) -> Result<Value, String> {
    let app_cfg = ctx.cfg.app(kind.name())?;
    let (sent, sent_file) = prepare(ctx, kind, run)?;
    let app = if run.attach {
        apps::attach(ctx, kind)?
    } else {
        launch::open(ctx, kind)?
    };
    println!("window: {}", win::describe(app.hwnd));
    let mut t = typing(ctx, &app, app_cfg.focus_class.as_deref(), run, &sent);
    sleep_ms(app_cfg.done_ms);
    let (raw, read_error) = read(ctx, &app, &mut t);
    let left_open = apps::clean_up(ctx, app);
    let received = raw.trim_end_matches(LINE_ENDS);
    let received_file = ctx.file(RECEIVED_FILE);
    out::write(&received_file, received)?;
    let diff = diff::compare(&sent, received, ctx.cfg.g1.context_chars);
    report(&t, run.count, &read_error, &diff, &left_open);
    Ok(json!({
        "gate": "G1", "app": kind.name(), "seed": run.seed, "count": run.count, "typed": t.typed,
        "pause_ms": run.pause_ms, "attached": run.attach,
        "stopped": t.stopped, "read_error": read_error, "focus": t.focus, "evidence": t.evidence,
        "trimmed_line_ends": raw.chars().count() - received.chars().count(),
        "diff": diff, "left_open": left_open,
        "files": { "sent": sent_file, "received": received_file },
    }))
}

/// Prints the first difference, if any.
fn print_diff(d: &Diff) {
    let verdict = if d.equal { "yes" } else { "no" };
    println!(
        "sent {} chars, received {}; equal: {verdict}",
        d.sent_len, d.received_len
    );
    if let Some(i) = d.first_diff {
        println!(
            "first difference at {i}: sent {} {:?}",
            d.sent_at, d.sent_context
        );
        println!(
            "                   received {} {:?}",
            d.received_at, d.received_context
        );
        println!("{} positions differ over the shorter length", d.mismatched);
    }
}

fn report(t: &Typing, count: usize, read_error: &Option<String>, d: &Diff, left_open: &[String]) {
    let note = t
        .stopped
        .as_ref()
        .map(|s| format!(" (stopped: {s})"))
        .unwrap_or_default();
    println!(
        "typed {}/{count}{note}; keyboard was on {}",
        t.typed, t.focus
    );
    if let Some(e) = read_error {
        println!("read back failed: {e}");
    }
    if !t.evidence.is_null() {
        println!("evidence: {}", t.evidence);
    }
    print_diff(d);
    for note in left_open {
        println!("{note}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_ok_needs_the_app_window_and_the_wanted_class() {
        assert!(focus_ok(true, "_WwG", Some("_WwG")));
        assert!(focus_ok(true, "Edit", Some("EDIT")));
        assert!(!focus_ok(true, "OpusApp", Some("_WwG")));
        assert!(!focus_ok(true, "", Some("_WwG")));
        assert!(focus_ok(true, "Chrome_WidgetWin_1", None));
        assert!(!focus_ok(false, "NUIDialog", None));
        assert!(!focus_ok(false, "_WwG", Some("_WwG")));
    }

    #[test]
    fn title_ok_needs_the_test_file_name_when_one_is_set() {
        let ours = "g1-notepad-1.txt";
        assert!(title_ok("g1-notepad-1.txt - Notepad", Some(ours)));
        assert!(!title_ok("*Some other updates - Notepad", Some(ours)));
        assert!(title_ok("Document1 - Word", None));
    }
}
