//! The quiet panic hook. A panic's message may quote what the user typed, so the hook logs only where it happened;
//! the kernel still catches module panics, which would otherwise print through Rust's own hook.

use std::panic::{self, Location};

/// Replaces Rust's panic hook for the whole process with `report`.
pub fn install() {
    panic::set_hook(Box::new(|info| report(info.location())));
}

/// Logs one error line naming the panic's place, `at`; never its message.
pub fn report(at: Option<&Location<'_>>) {
    if let Some(at) = at {
        let (file, line, column) = (at.file(), at.line(), at.column());
        tracing::error!(file, line, column, "a panic happened");
    } else {
        tracing::error!("a panic happened at an unknown place");
    }
}
