//! The quiet panic hook: it first has the mouse hook let go, so no button stays down, then logs only where the panic
//! happened, as its message may quote what the user typed. The kernel still catches module panics, which would
//! otherwise print through Rust's own hook.

use std::io::{self, Write};
use std::panic::{self, Location};

/// What standard error says before the place.
const PANIC_AT: &str = "KeyXtend: a panic happened at";
/// The place when Rust gives none.
const UNKNOWN: &str = "an unknown place";

/// Replaces Rust's panic hook for the whole process with `release`, which lets go of anything the mouse hook holds, then
/// `report` to standard error. `release` must never panic; it may run on the hook's own thread.
pub fn install(release: fn()) {
    panic::set_hook(Box::new(move |info| {
        release();
        report(info.location(), &mut io::stderr());
    }));
}

/// Logs one error line naming the panic's place, `at`; never its message. Until the run's log is set up, it also
/// writes the place to `out`.
pub fn report(at: Option<&Location<'_>>, out: &mut impl Write) {
    if let Some(at) = at {
        let (file, line, column) = (at.file(), at.line(), at.column());
        tracing::error!(file, line, column, "a panic happened");
    } else {
        tracing::error!("a panic happened at {UNKNOWN}");
    }
    if !crate::log::installed() {
        let place = at.map_or_else(|| UNKNOWN.to_owned(), Location::to_string);
        let _ = writeln!(out, "{PANIC_AT} {place}");
    }
}
