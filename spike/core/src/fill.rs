//! Quick-fill: after Windows Hello says yes, types a fake test value into the app that was in front.

use std::sync::{Arc, Mutex};

use crate::com::Com;
use crate::config::SpikeConfig;
use crate::hello::{self, Answer};
use crate::{inject, window};

/// The latest fill note, for the UI to show on its next tick.
pub type Note = Arc<Mutex<Option<String>>>;

/// Starts every fill note that carries an error.
const NOTE_PREFIX: &str = "fill: ";
/// Shown while Hello's prompt is open.
const ASKING: &str = "fill: asking Windows Hello";
/// Our keyboard window, which owns the prompt, was not found.
const NO_OWNER: &str = "fill: our window was not found";
/// Typed after Verified.
const TYPED: &str = "fill: typed";
/// Hello closed.
const CANCELED: &str = "fill: Hello canceled, nothing typed";
/// Hello off or refused.
const UNAVAILABLE: &str = "fill: Hello unavailable, nothing typed";
/// The app in front did not come back.
const MOVED: &str = "fill: the app in front changed, nothing typed";
/// No app of someone else's was in front.
const NO_TARGET: &str = "fill: no app in front, nothing typed";

/// One fill: our window that owns the prompt, the app in front, and what to type.
struct Fill {
    /// Our window, as a raw handle value so the request can cross threads.
    owner: isize,
    /// The app in front when the button was clicked.
    target: isize,
    /// Hello's prompt text.
    message: String,
    /// The fake value; never logged.
    value: String,
    /// Wait after bringing the target back, in ms.
    settle_ms: u64,
}

/// Runs `f` on its own thread, so the UI never waits for Hello; the result lands in `note`.
fn spawn(f: Fill, note: Note) {
    std::thread::spawn(move || {
        let said = run(&f);
        if let Ok(mut n) = note.lock() {
            *n = Some(said);
        }
    });
}

/// Starts a fill of `value` into the app in front, with Hello's prompt owned by `face`'s window; returns the note to show now.
pub fn start(cfg: &SpikeConfig, face: &str, value: String, note: &Note) -> &'static str {
    let Some(owner) = window::own_titled(&cfg.title(face)) else {
        return NO_OWNER;
    };
    let f = Fill {
        owner: owner.0 as isize,
        target: window::foreground().0 as isize,
        message: cfg.tools.hello_message.clone(),
        value,
        settle_ms: cfg.tools.fill_settle_ms,
    };
    spawn(f, Arc::clone(note));
    ASKING
}

/// Takes the latest note, if there is one.
pub fn take(note: &Note) -> Option<String> {
    note.lock().ok().and_then(|mut n| n.take())
}

/// Ok means type now; otherwise the note says why not. `back` runs only after Verified.
fn decide(a: Answer, back: impl FnOnce() -> bool) -> Result<(), &'static str> {
    match a {
        Answer::Canceled => Err(CANCELED),
        Answer::Unavailable => Err(UNAVAILABLE),
        Answer::Verified if back() => Ok(()),
        Answer::Verified => Err(MOVED),
    }
}

/// Asks Hello, brings the target back, then types; returns the note.
fn run(f: &Fill) -> String {
    let (owner, target) = (window::from_raw(f.owner), window::from_raw(f.target));
    if target.is_invalid() || window::is_ours(target) {
        return NO_TARGET.to_string();
    }
    // WinRT needs COM, and windows-rs keeps Hello's factory for the whole process, so COM stays on.
    if let Err(e) = Com::start().map(Com::keep) {
        return format!("{NOTE_PREFIX}{e}");
    }
    let answer = match hello::ask(owner, &f.message) {
        Ok(a) => a,
        Err(e) => return e,
    };
    match decide(answer, || window::bring_back(target, f.settle_ms)) {
        Ok(()) => inject::text(&f.value).map_or_else(|e| e, |()| TYPED.to_string()),
        Err(why) => why.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn only_verified_with_the_app_back_in_front_types() {
        assert_eq!(decide(Answer::Verified, || true), Ok(()));
        assert_eq!(decide(Answer::Verified, || false), Err(MOVED));
    }

    #[test]
    fn cancel_and_unavailable_type_nothing_and_leave_the_focus_alone() {
        let asked = Cell::new(false);
        let back = || {
            asked.set(true);
            true
        };
        assert_eq!(decide(Answer::Canceled, back), Err(CANCELED));
        assert_eq!(decide(Answer::Unavailable, back), Err(UNAVAILABLE));
        assert!(!asked.get(), "focus is only moved after Verified");
    }

    #[test]
    fn a_note_is_taken_once() {
        let note: Note = Arc::default();
        *note.lock().expect("lock") = Some("x".into());
        assert_eq!(take(&note).as_deref(), Some("x"));
        assert_eq!(take(&note), None);
    }
}
