//! One running copy per user session. The first copy claims the session and runs; a later copy hands its command to
//! the running one, trying again for a while, then ends. Quit with no copy running starts nothing.

use kx_platform::ui::{Claim, Command, Instance, Running, UiError};
use kx_ui::UiSettings;
use std::time::Duration;

/// How a later copy reaches the running one: how many tries, and the pause between two tries.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Reach {
    /// Tries before it gives up.
    pub tries: u32,
    /// The pause between two tries.
    pub wait: Duration,
}

impl Reach {
    /// The reach `settings` give (`[ui] send_tries` and `send_wait_ms`).
    #[must_use]
    pub fn of(settings: &UiSettings) -> Self {
        Self {
            tries: settings.send_tries,
            wait: Duration::from_millis(settings.send_wait_ms),
        }
    }
}

/// What this copy does next.
pub enum Start {
    /// It is the running copy: it holds the session until the claim drops.
    Run(Box<dyn Claim>),
    /// Its work is done: the running copy took the command, or Quit found none running.
    Done,
    /// Another copy holds the session but never took the command, for the reason given.
    Unanswered(UiError),
}

/// Claims the session for `command`, or hands it to the copy that holds it; `pause` waits between tries.
pub fn start(
    instance: &dyn Instance,
    command: Command,
    reach: Reach,
    pause: impl FnMut(Duration),
) -> Start {
    match instance.claim() {
        Ok(_) if command == Command::Quit => Start::Done,
        Ok(claim) => Start::Run(claim),
        Err(Running) => match send(instance, command, reach, pause) {
            Ok(()) => Start::Done,
            Err(e) => Start::Unanswered(e),
        },
    }
}

/// Sends `command` to the running copy until it takes it or `reach.tries` run out, pausing between two tries.
///
/// # Errors
/// The last try's `UiError`, or `NoWindow` when `reach` allows no try.
pub fn send(
    instance: &dyn Instance,
    command: Command,
    reach: Reach,
    mut pause: impl FnMut(Duration),
) -> Result<(), UiError> {
    let mut last = UiError::NoWindow;
    for tried in 0..reach.tries {
        if tried > 0 {
            pause(reach.wait);
        }
        match instance.send(command) {
            Ok(()) => return Ok(()),
            Err(e) => last = e,
        }
    }
    Err(last)
}

#[cfg(test)]
mod tests;
