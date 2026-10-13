//! One running copy per user session. The first copy claims the session and runs; a later copy hands its command to
//! the running one, trying again for a while, and runs itself if that copy quits meanwhile, while Quit with no copy
//! running starts nothing.

use kx_module_api::SettingsError;
use kx_platform::ui::{Claim, Command, Instance, Running, UiError};
use kx_settings::peek_section;
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
    /// The reach the `[ui]` section of the user's settings `file` gives, read without changing it; the defaults when
    /// there is no file or the section does not check.
    ///
    /// # Errors
    /// When the compiled-in `[ui]` defaults do not read.
    pub fn from_file(file: Option<&toml::Table>) -> Result<Self, SettingsError> {
        let settings = peek_section::<UiSettings>(file, kx_ui::ID, &kx_ui::SPEC)?;
        Ok(Self::of(&settings))
    }

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

/// Claims the session for `command`, or hands it to the copy that holds it, trying at least once and pausing with
/// `pause` between tries. Each try claims again first, so a copy that quits meanwhile leaves the session to this one.
pub fn start(
    instance: &dyn Instance,
    command: Command,
    reach: Reach,
    mut pause: impl FnMut(Duration),
) -> Start {
    let mut last = UiError::NoWindow;
    for tried in 0..reach.tries.max(1) {
        if tried > 0 {
            pause(reach.wait);
        }
        match instance.claim() {
            Ok(_) if command == Command::Quit => return Start::Done,
            Ok(claim) => return Start::Run(claim),
            Err(Running) => match instance.send(command) {
                Ok(()) => return Start::Done,
                Err(e) => last = e,
            },
        }
    }
    Start::Unanswered(last)
}

#[cfg(test)]
mod tests;
