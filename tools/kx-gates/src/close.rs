//! `close`: the product gets Quit on its command window, as its own `--quit` sends it, and the gate waits for its
//! process to end; nothing is started. The frozen spike face only gets `WM_CLOSE`, which leaves its process running.
#![cfg(windows)]

use kx_platform::ui::{Command, Instance, UiError};
use kx_platform_windows::WindowsPlatform;
use serde_json::{Value, json};

use crate::apps::Ctx;
use crate::proc::Process;
use crate::product::{self, NAME};
use crate::{mouse, win, winfind};

/// Runs `close` on `name`: the product or the frozen face.
pub fn run(ctx: &Ctx, name: &str) -> Result<Value, String> {
    if name == NAME {
        let (exe, ended) = quit_running(ctx)?;
        Ok(result(&exe, ended))
    } else {
        close_face(ctx, name)
    }
}

/// The result line of `close` on the product, run from `program`.
fn result(program: &str, ended: bool) -> Value {
    json!({ "gate": "close", "keyboard": NAME, "program": program, "pass": ended })
}

/// Why the running copy did not take Quit.
fn why_not(e: UiError) -> String {
    match e {
        UiError::NoWindow => {
            format!("{NAME} runs but has no command window yet; try again once it shows")
        }
        UiError::Refused(_) => {
            format!("{NAME} did not take Quit: {e}; run the installed uiAccess kx-gates")
        }
    }
}

/// Posts Quit to the copy that holds the session in `instance`.
fn ask_quit(instance: &dyn Instance) -> Result<(), String> {
    instance.send(Command::Quit).map_err(why_not)
}

/// Posts Quit to `instance` up to `tries` times, calling `wait` between them, as a copy just started may not listen
/// yet; it stops once `gone` says the copy ended. The tries it took; a refusal is final.
fn ask_quit_tries(
    instance: &dyn Instance,
    tries: u32,
    gone: impl Fn() -> bool,
    mut wait: impl FnMut(),
) -> Result<u32, String> {
    let mut n = 1;
    loop {
        if gone() {
            return Ok(n - 1);
        }
        match instance.send(Command::Quit) {
            Ok(()) => return Ok(n),
            Err(UiError::NoWindow) if n < tries => wait(),
            Err(e) => return Err(why_not(e)),
        }
        n += 1;
    }
}

/// True when `process` ended within `[timing] quit_wait_ms`.
fn ended_within(ctx: &Ctx, process: &Process) -> bool {
    let t = &ctx.cfg.timing;
    win::poll_until(t.quit_wait_ms, t.poll_ms, || process.ended().then_some(())).is_some()
}

/// Posts Quit to a copy G8 just started, trying `[g8] quit_tries` times as its command window may not exist yet;
/// whether `process` ended within `[timing] quit_wait_ms`.
pub fn quit_started(ctx: &Ctx, process: &Process) -> Result<bool, String> {
    let g = &ctx.cfg.g8;
    let platform = WindowsPlatform::new().map_err(|e| e.to_string())?;
    let gone = || process.ended();
    ask_quit_tries(&*platform.instance(), g.quit_tries, gone, || {
        win::sleep_ms(g.quit_try_ms);
    })?;
    Ok(ended_within(ctx, process))
}

/// Posts Quit to the running product, which starts nothing; its program, and whether it ended within
/// `[timing] quit_wait_ms`.
pub fn quit_running(ctx: &Ctx) -> Result<(String, bool), String> {
    let not_running = || format!("{NAME} is not running");
    let (pid, exe) = product::running().ok_or_else(not_running)?;
    // Opened before the quit, so the id cannot name another process by the time we look.
    let process = Process::to_wait(pid)?;
    let platform = WindowsPlatform::new().map_err(|e| e.to_string())?;
    ask_quit(&*platform.instance())?;
    Ok((exe, ended_within(ctx, &process)))
}

/// Closes the frozen face `name` and waits until its window is gone; needs the uiAccess kx-gates.
fn close_face(ctx: &Ctx, name: &str) -> Result<Value, String> {
    let hwnd = mouse::find_face(ctx.spike, name)?;
    win::close(hwnd)?;
    let t = &ctx.cfg.timing;
    let closed = winfind::wait_gone(hwnd, t.close_wait_ms, t.poll_ms);
    Ok(json!({ "gate": "close", "face": name, "pass": closed }))
}

#[cfg(test)]
mod tests {
    use kx_platform_fake::FakeInstance;

    use super::*;

    #[test]
    fn quit_is_posted_to_the_running_copy_and_starts_nothing() {
        let session = FakeInstance::new();
        let mut claim = session.claim().expect("the first copy runs");
        claim.listen(Box::new(|_| {})).expect("it listens");
        assert_eq!(ask_quit(&session.clone()), Ok(()));
        assert_eq!(session.sent(), [Command::Quit]);
    }

    #[test]
    fn a_copy_just_started_gets_quit_once_it_listens() {
        let session = FakeInstance::new();
        let late = std::cell::RefCell::new(Vec::new());
        let waits = std::cell::Cell::new(0);
        let wait = || {
            waits.set(waits.get() + 1);
            if waits.get() == 2 {
                let mut claim = session.claim().expect("the started copy runs");
                claim.listen(Box::new(|_| {})).expect("it listens");
                late.borrow_mut().push(claim);
            }
        };
        assert_eq!(ask_quit_tries(&session.clone(), 5, || false, wait), Ok(3));
        assert_eq!(session.sent(), [Command::Quit]);
        let never = ask_quit_tries(&FakeInstance::new(), 3, || false, || {});
        assert!(
            never.is_err_and(|e| e.contains("command window")),
            "it never listened"
        );
    }

    #[test]
    fn a_copy_that_ended_by_itself_gets_no_more_tries() {
        let session = FakeInstance::new();
        let gone = std::cell::Cell::new(false);
        let tries = ask_quit_tries(&session.clone(), 5, || gone.get(), || gone.set(true));
        assert_eq!(tries, Ok(1), "one try, then it was gone");
        assert!(session.sent().is_empty());
    }

    #[test]
    fn a_copy_that_cannot_hear_quit_says_why() {
        let none = ask_quit(&FakeInstance::new()).expect_err("no copy listens");
        assert!(none.contains("command window"), "{none}");
        let refused = why_not(UiError::Refused(5));
        assert!(
            refused.contains("code 5") && refused.contains("uiAccess"),
            "{refused}"
        );
    }

    #[test]
    fn the_result_names_the_product_as_its_keyboard() {
        let v = result(r"C:\x\keyxtend.exe", true);
        assert_eq!(v["keyboard"], product::NAME);
        assert_eq!(
            (v["gate"].as_str(), v["pass"].as_bool()),
            (Some("close"), Some(true))
        );
    }
}
