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
    listen_error: Option<UiError>,
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

    /// Makes later `listen` calls fail with `error`, or work again with `None`.
    pub fn set_listen_error(&self, error: Option<UiError>) {
        lock(&self.0).listen_error = error;
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

    /// Reaches the running copy only once it listens, as the real adapter finds it by the window it listens on.
    /// It calls the listener while holding the session, so a listener must not send.
    fn send(&self, command: Command) -> Result<(), UiError> {
        let mut session = lock(&self.0);
        let Some(listener) = &session.listener else {
            return Err(UiError::NoWindow);
        };
        listener(command);
        session.sent.push(command);
        Ok(())
    }
}

/// The running copy's claim; dropping it ends the listener and frees the session.
struct FakeClaim(Arc<Mutex<Session>>);

impl Claim for FakeClaim {
    fn listen(&mut self, on: Box<dyn Fn(Command) + Send>) -> Result<(), UiError> {
        let mut session = lock(&self.0);
        if let Some(error) = session.listen_error {
            return Err(error);
        }
        session.listener = Some(on);
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
    fn a_running_copy_that_does_not_listen_yet_has_no_window() {
        let running = FakeInstance::new();
        let later = running.clone();
        let _claim = running.claim().unwrap();
        assert_eq!(later.send(Command::Show), Err(UiError::NoWindow));
        assert!(running.sent().is_empty());
    }

    #[test]
    fn a_later_listen_replaces_the_first_and_a_refused_one_keeps_it() {
        let running = FakeInstance::new();
        let heard = Arc::new(Mutex::new(Vec::new()));
        let hear = |tag: &'static str| {
            let keep = Arc::clone(&heard);
            Box::new(move |c: Command| keep.lock().unwrap().push((tag, c)))
        };
        let mut claim = running.claim().unwrap();
        claim.listen(hear("first")).unwrap();
        claim.listen(hear("second")).unwrap();
        running.set_listen_error(Some(UiError::Refused(9)));
        assert_eq!(claim.listen(hear("third")), Err(UiError::Refused(9)));
        running.send(Command::Show).unwrap();
        assert_eq!(*heard.lock().unwrap(), [("second", Command::Show)]);
    }

    #[test]
    fn a_refused_listen_leaves_no_window() {
        let running = FakeInstance::new();
        running.set_listen_error(Some(UiError::Refused(8)));
        let mut claim = running.claim().unwrap();
        assert_eq!(claim.listen(Box::new(|_| {})), Err(UiError::Refused(8)));
        assert_eq!(running.send(Command::Quit), Err(UiError::NoWindow));
    }

    #[test]
    fn with_no_running_copy_a_send_finds_no_window() {
        let fake = FakeInstance::new();
        assert_eq!(fake.send(Command::Show), Err(UiError::NoWindow));
        assert!(fake.sent().is_empty());
    }
}
