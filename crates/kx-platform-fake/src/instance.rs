//! A fake of the one-running-copy port: copies share one fake session.

use crate::lock;
use kx_platform::ui::{Claim, Command, Instance, Running, UiError};
use std::fmt;
use std::sync::{Arc, Mutex};

/// What the copies of one fake session share.
#[derive(Default)]
struct Session {
    claimed: bool,
    listener: Option<Box<dyn Fn(Command) + Send>>,
    sent: Vec<Command>,
}

/// A fake user session; each clone is another copy of the app in it.
#[derive(Clone, Default)]
pub struct FakeInstance(Arc<Mutex<Session>>);

impl FakeInstance {
    /// A session where no copy runs yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every command that reached the running copy, oldest first.
    #[must_use]
    pub fn sent(&self) -> Vec<Command> {
        lock(&self.0).sent.clone()
    }
}

impl fmt::Debug for FakeInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session = lock(&self.0);
        f.debug_struct("FakeInstance")
            .field("claimed", &session.claimed)
            .field("sent", &session.sent)
            .finish_non_exhaustive()
    }
}

impl Instance for FakeInstance {
    fn claim(&self) -> Result<Box<dyn Claim>, Running> {
        let mut session = lock(&self.0);
        if session.claimed {
            return Err(Running);
        }
        session.claimed = true;
        Ok(Box::new(FakeClaim(Arc::clone(&self.0))))
    }

    /// Hands `command` to the listener while holding the session, so a listener must not send.
    fn send(&self, command: Command) -> Result<(), UiError> {
        let mut session = lock(&self.0);
        if !session.claimed {
            return Err(UiError::NoWindow);
        }
        session.sent.push(command);
        if let Some(listener) = &session.listener {
            listener(command);
        }
        Ok(())
    }
}

/// The running copy's claim; dropping it ends the listener and frees the session.
struct FakeClaim(Arc<Mutex<Session>>);

impl Claim for FakeClaim {
    fn listen(&mut self, on: Box<dyn Fn(Command) + Send>) -> Result<(), UiError> {
        lock(&self.0).listener = Some(on);
        Ok(())
    }
}

impl Drop for FakeClaim {
    fn drop(&mut self) {
        let mut session = lock(&self.0);
        session.claimed = false;
        session.listener = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_one_copy_claims_until_it_lets_go() {
        let first = FakeInstance::new();
        let second = first.clone();
        let claim = first.claim().unwrap();
        assert!(matches!(second.claim(), Err(Running)));
        drop(claim);
        assert!(second.claim().is_ok());
    }

    #[test]
    fn a_later_copy_reaches_the_listener() {
        let running = FakeInstance::new();
        let later = running.clone();
        let heard = Arc::new(Mutex::new(Vec::new()));
        let keep = Arc::clone(&heard);
        let mut claim = running.claim().unwrap();
        claim
            .listen(Box::new(move |c| keep.lock().unwrap().push(c)))
            .unwrap();
        assert_eq!(later.send(Command::Show), Ok(()));
        assert_eq!(later.send(Command::Quit), Ok(()));
        assert_eq!(*heard.lock().unwrap(), [Command::Show, Command::Quit]);
        assert_eq!(running.sent(), [Command::Show, Command::Quit]);
    }

    #[test]
    fn with_no_running_copy_a_send_finds_no_window() {
        let fake = FakeInstance::new();
        assert_eq!(fake.send(Command::Show), Err(UiError::NoWindow));
        assert!(fake.sent().is_empty());
    }
}
