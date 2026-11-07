//! A fake web link: it records each link it was asked to open, refuses while the test sets an error, holds its calls
//! while told, and lets a test wait for a call made on another thread without sleeping.

use crate::lock;
use kx_module_api::ports::{Link, LinkError, WebLink};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// What the fake was asked and how it answers.
#[derive(Debug, Default)]
struct State {
    /// Every link asked, oldest first.
    opened: Vec<Link>,
    /// The calls that returned.
    left: usize,
    /// The error later calls give.
    error: Option<LinkError>,
    /// Calls wait inside `open` while this holds.
    held: bool,
}

/// A web link that keeps every link asked, refused ones too; its `Debug` prints them redacted.
#[derive(Debug, Default)]
pub struct FakeWebLink {
    state: Mutex<State>,
    changed: Condvar,
}

impl FakeWebLink {
    /// A web link with no call yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every link asked so far, oldest first.
    #[must_use]
    pub fn opened(&self) -> Vec<Link> {
        lock(&self.state).opened.clone()
    }

    /// Waits up to `within` until `n` links were asked, so a test never sleeps; true when they were.
    #[must_use]
    pub fn wait_opened(&self, n: usize, within: Duration) -> bool {
        self.wait(within, |s| s.opened.len() >= n)
    }

    /// Waits up to `within` until `n` calls returned; true when they did.
    #[must_use]
    pub fn wait_left(&self, n: usize, within: Duration) -> bool {
        self.wait(within, |s| s.left >= n)
    }

    /// Refuses later calls with `error`, or takes them again with `None`; a debug build stops on one `open` never
    /// returns.
    pub fn set_error(&self, error: Option<LinkError>) {
        let ported = matches!(
            error,
            None | Some(LinkError::Unavailable | LinkError::Refused(_))
        );
        debug_assert!(ported, "the port never returns {error:?} from open");
        lock(&self.state).error = error;
    }

    /// Holds calls inside `open` after they are recorded, as a slow shell would, until called with false.
    pub fn hold(&self, on: bool) {
        lock(&self.state).held = on;
        self.changed.notify_all();
    }

    /// Waits up to `within` until `done` holds for the state.
    fn wait(&self, within: Duration, done: impl Fn(&State) -> bool) -> bool {
        let waited = self
            .changed
            .wait_timeout_while(lock(&self.state), within, |s| !done(s));
        let (state, _) = waited.unwrap_or_else(PoisonError::into_inner);
        done(&state)
    }

    /// Waits while calls are held.
    fn wait_released<'a>(&self, state: MutexGuard<'a, State>) -> MutexGuard<'a, State> {
        let released = self.changed.wait_while(state, |s| s.held);
        released.unwrap_or_else(PoisonError::into_inner)
    }
}

impl WebLink for FakeWebLink {
    fn open(&self, link: &Link) -> Result<(), LinkError> {
        let mut state = lock(&self.state);
        state.opened.push(link.clone());
        self.changed.notify_all();
        let mut state = self.wait_released(state);
        state.left += 1;
        self.changed.notify_all();
        state.error.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::LINK_SCHEME;
    use std::sync::Arc;

    /// How long a test waits for another thread's call.
    const WAIT: Duration = Duration::from_secs(5);

    /// A checked link to `path` on a test host.
    fn link(path: &str) -> Link {
        Link::new(format!("{LINK_SCHEME}example.org/{path}")).unwrap()
    }

    #[test]
    fn it_records_each_link_and_refuses_while_told() {
        let web = FakeWebLink::new();
        assert_eq!(web.open(&link("a")), Ok(()));
        web.set_error(Some(LinkError::Refused(5)));
        assert_eq!(web.open(&link("b")), Err(LinkError::Refused(5)));
        web.set_error(Some(LinkError::Unavailable));
        assert_eq!(web.open(&link("c")), Err(LinkError::Unavailable));
        web.set_error(None);
        assert_eq!(web.open(&link("d")), Ok(()));
        let want = ["a", "b", "c", "d"].map(link);
        assert_eq!(web.opened(), want, "refused calls count too");
    }

    #[test]
    fn a_test_waits_for_a_call_from_another_thread() {
        let web = Arc::new(FakeWebLink::new());
        assert!(!web.wait_opened(1, Duration::ZERO), "nothing asked yet");
        let there = Arc::clone(&web);
        let call = std::thread::spawn(move || there.open(&link("e")));
        assert!(web.wait_opened(1, WAIT));
        assert_eq!(call.join().ok(), Some(Ok(())));
        assert_eq!(web.opened(), [link("e")]);
    }

    #[test]
    fn a_held_call_is_recorded_and_returns_only_when_released() {
        let web = Arc::new(FakeWebLink::new());
        web.hold(true);
        let there = Arc::clone(&web);
        let call = std::thread::spawn(move || there.open(&link("f")));
        assert!(web.wait_opened(1, WAIT));
        assert!(!web.wait_left(1, Duration::ZERO), "still held");
        web.hold(false);
        assert!(web.wait_left(1, WAIT));
        assert_eq!(call.join().ok(), Some(Ok(())));
    }

    #[test]
    fn debug_never_prints_a_link() {
        let web = FakeWebLink::new();
        assert_eq!(web.open(&link("secret-words")), Ok(()));
        let text = format!("{web:?}");
        assert!(text.contains("left: 1"), "{text}");
        assert!(
            !text.contains("secret-words") && !text.contains(LINK_SCHEME),
            "{text}"
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "never returns")]
    fn an_error_the_port_never_returns_from_open_is_a_test_mistake() {
        FakeWebLink::new().set_error(Some(LinkError::BadChar));
    }
}
