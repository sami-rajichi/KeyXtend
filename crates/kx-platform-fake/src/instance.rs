//! A fake of the one-running-copy port: copies share one fake session.
//! Like the adapter, the running copy hears commands in order, one at a time, outside the lock.

use crate::lock;
use kx_platform::ui::{Claim, Command, Instance, Running, UiError};
use std::collections::VecDeque;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

/// Hears the commands a later copy sends.
type OnCommand = Box<dyn Fn(Command) + Send>;

/// What the copies of one fake session share.
#[derive(Default)]
struct Session {
    /// The claim that holds the session, by number; `None` while no copy runs.
    holder: Option<u64>,
    /// How many claims were made; it numbers the next one.
    claims: u64,
    /// The running copy listens, as the adapter's copy has its command window.
    listening: bool,
    listener: Option<OnCommand>,
    /// Commands still to hear, oldest first.
    queue: VecDeque<Command>,
    /// The thread hearing commands now, if any.
    hearing: Option<ThreadId>,
    /// How many claim drops wait for that thread, so a test can see a drop wait.
    waiting: usize,
    sent: Vec<Command>,
    listen_error: Option<UiError>,
}

/// The session, and its signal: hearing stopped, or a claim drop began to wait for it.
#[derive(Default)]
struct Shared {
    session: Mutex<Session>,
    idle: Condvar,
}

/// A fake user session; each clone is another copy of the app in it.
#[derive(Clone, Default)]
pub struct FakeInstance(Arc<Shared>);

impl FakeInstance {
    /// A session where no copy runs yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes later `listen` calls fail with `error`, or work again with `None`.
    pub fn set_listen_error(&self, error: Option<UiError>) {
        lock(&self.0.session).listen_error = error;
    }

    /// Every command that reached the running copy, oldest first.
    #[must_use]
    pub fn sent(&self) -> Vec<Command> {
        lock(&self.0.session).sent.clone()
    }
}

impl fmt::Debug for FakeInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let session = lock(&self.0.session);
        f.debug_struct("FakeInstance")
            .field("claimed", &session.holder.is_some())
            .field("sent", &session.sent)
            .finish_non_exhaustive()
    }
}

impl Instance for FakeInstance {
    fn claim(&self) -> Result<Box<dyn Claim>, Running> {
        let mut session = lock(&self.0.session);
        if session.holder.is_some() {
            return Err(Running);
        }
        session.claims += 1;
        session.holder = Some(session.claims);
        let (shared, id) = (Arc::clone(&self.0), session.claims);
        Ok(Box::new(FakeClaim { shared, id }))
    }

    /// Reaches the running copy only once it listens, as the adapter finds it by its window.
    /// A command sent while another is heard, from any thread, waits its turn.
    fn send(&self, command: Command) -> Result<(), UiError> {
        let mut session = lock(&self.0.session);
        if !session.listening {
            return Err(UiError::NoWindow);
        }
        session.sent.push(command);
        session.queue.push_back(command);
        if session.hearing.is_none() {
            session.hearing = Some(thread::current().id());
            hear(&self.0, session);
        }
        Ok(())
    }
}

/// Hears each waiting command outside the lock, then wakes a drop that waits; a listener set meanwhile wins.
fn hear<'a>(shared: &'a Shared, mut session: MutexGuard<'a, Session>) {
    while let Some(command) = session.queue.pop_front() {
        let holder = session.holder;
        let Some(on) = session.listener.take() else {
            break;
        };
        drop(session);
        let _ = catch_unwind(AssertUnwindSafe(|| on(command)));
        let mut now = lock(&shared.session);
        let stale = if now.holder == holder && now.listener.is_none() {
            now.listener = Some(on);
            None
        } else {
            Some(on)
        };
        drop(now);
        drop(stale);
        session = lock(&shared.session);
    }
    session.hearing = None;
    shared.idle.notify_all();
}

/// The running copy's claim; dropping it waits for a command being heard, then frees the session.
struct FakeClaim {
    shared: Arc<Shared>,
    id: u64,
}

impl Claim for FakeClaim {
    fn listen(&mut self, on: OnCommand) -> Result<(), UiError> {
        let mut session = lock(&self.shared.session);
        if let Some(error) = session.listen_error {
            return Err(error);
        }
        let earlier = session.listener.replace(on);
        session.listening = true;
        drop(session);
        drop(earlier);
        Ok(())
    }
}

impl Drop for FakeClaim {
    /// Waits while another thread hears a command, as the adapter joins its listener thread.
    fn drop(&mut self) {
        let me = thread::current().id();
        let mut session = lock(&self.shared.session);
        let must_wait = session.hearing.is_some_and(|t| t != me);
        if must_wait {
            session.waiting += 1;
            self.shared.idle.notify_all();
        }
        while session.hearing.is_some_and(|t| t != me) {
            let waited = self.shared.idle.wait(session);
            session = waited.unwrap_or_else(PoisonError::into_inner);
        }
        if must_wait {
            session.waiting -= 1;
        }
        if session.holder != Some(self.id) {
            return;
        }
        session.holder = None;
        session.listening = false;
        session.queue.clear();
        let gone = session.listener.take();
        drop(session);
        drop(gone);
    }
}

#[cfg(test)]
mod tests;
