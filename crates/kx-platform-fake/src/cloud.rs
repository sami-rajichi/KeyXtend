//! A fake cloud speech service: each run tells the answers the test scripted for its pieces, in order, and holds the
//! rest until the test answers or cuts them; it counts the pieces by size and wipes their sound and the key at once.

use crate::lock;
use kx_module_api::Secret;
use kx_module_api::ports::{CloudAsk, CloudFault, CloudNote, CloudRun, CloudSpeech, CloudTest};
use std::fmt;
use std::sync::{Arc, PoisonError};
use std::time::Duration;

mod calls;
mod log;
pub mod suite;

pub use calls::{CloudAnswer, CloudCalls, CloudLog, CloudState, CloudTestLog};
use log::{FakeCloudRun, Shared};

/// The callback a run tells its notes to.
type Tell = Box<dyn Fn(CloudNote) + Send>;

/// True when a run may tell `fault`: `Range` is only `send`'s own refusal.
fn may_tell(fault: CloudFault) -> bool {
    !matches!(fault, CloudFault::Range(_))
}

/// True when every fault in `answers` is one a run may tell.
fn ported(answers: &[CloudAnswer]) -> bool {
    answers.iter().all(|a| match a {
        CloudAnswer::Fault(fault) => may_tell(*fault),
        CloudAnswer::Text(_) => true,
    })
}

/// Cloud speech shared by its clones: a run's notes are told in order on the thread that sends, answers or cuts it,
/// and a `tell` must not call back into the fake or drop its run.
#[derive(Clone, Default)]
pub struct FakeCloud {
    shared: Arc<Shared>,
}

impl FakeCloud {
    /// Cloud speech with no run and nothing scripted.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Scripts the answers to the next unscripted run's pieces, in order; a run holds the pieces past its script, or
    /// all of them with none, until `answer`. A debug build stops on a `Range` fault, which a run never tells.
    pub fn script(&self, answers: Vec<CloudAnswer>) {
        debug_assert!(ported(&answers), "a run never tells a Range fault");
        lock(&self.shared.log).scripts.push_back(answers.into());
    }

    /// Answers the held pieces of the run at `run` in `calls().runs` with `answers`, on this thread; false when it
    /// told nothing, as once that run ended or was dropped. Answers past its last piece are dropped.
    #[must_use]
    pub fn answer(&self, run: usize, answers: Vec<CloudAnswer>) -> bool {
        debug_assert!(ported(&answers), "a run never tells a Range fault");
        self.shared.tell(run, |log| log.take(run, answers))
    }

    /// Cuts the run at `run` while it sends, as a dropped connection does: it tells `Fault(fault)` and nothing after;
    /// false when it was not sending.
    #[must_use]
    pub fn cut(&self, run: usize, fault: CloudFault) -> bool {
        debug_assert!(may_tell(fault), "a run never tells {fault:?}");
        self.shared.tell(run, |log| log.cut(run, fault))
    }

    /// Fails the next `send` with `fault`, once, opening no run; a debug build stops on anything but `Gone`, as a
    /// `Range` comes only from the ask's own check.
    pub fn fail(&self, fault: CloudFault) {
        let ported = matches!(fault, CloudFault::Gone);
        debug_assert!(ported, "send never returns {fault:?} unasked");
        lock(&self.shared.log).fail = Some(fault);
    }

    /// Scripts the next key test's answer, after those queued; a test with none answers `Ok`. A debug build stops on
    /// a `Range`, which comes only from the test's own check.
    pub fn script_test(&self, answer: Result<(), CloudFault>) {
        let ported = !matches!(answer, Err(CloudFault::Range(_)));
        debug_assert!(ported, "a key test never answers {answer:?} unasked");
        lock(&self.shared.log).test_answers.push_back(answer);
    }

    /// Holds key tests inside `test` once recorded, as a slow service does, until called with false.
    pub fn hold_tests(&self, on: bool) {
        lock(&self.shared.log).held = on;
        self.shared.changed.notify_all();
    }

    /// Sets the key the service takes: for any other, a run tells only `Fault(Key)` and a test answers `Key`, both
    /// leaving the scripts queued; `None` takes any. It is the test's own key; a sent key is never kept.
    pub fn take_key(&self, key: Option<Secret>) {
        lock(&self.shared.log).key = key;
    }

    /// Every run and key test so far, oldest first.
    #[must_use]
    pub fn calls(&self) -> CloudCalls {
        lock(&self.shared.log).calls.clone()
    }

    /// Waits up to `within` until `until` holds for the calls, so a test never sleeps; true when it does.
    #[must_use]
    pub fn wait(&self, within: Duration, until: impl Fn(&CloudCalls) -> bool) -> bool {
        let log = lock(&self.shared.log);
        let waited = (self.shared.changed).wait_timeout_while(log, within, |l| !until(&l.calls));
        let (log, _) = waited.unwrap_or_else(PoisonError::into_inner);
        until(&log.calls)
    }
}

/// Shows the counts, never a key, a text or the sound.
impl fmt::Debug for FakeCloud {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log = lock(&self.shared.log);
        let runs = &log.calls.runs;
        let sending = runs.iter().filter(|r| r.state == CloudState::Sending);
        f.debug_struct("FakeCloud")
            .field("runs", &runs.len())
            .field("sending", &sending.count())
            .field("tests", &log.calls.tests.len())
            .finish_non_exhaustive()
    }
}

impl CloudSpeech for FakeCloud {
    /// Checks `ask` by its own rule, records it without its key or sound, then tells its scripted answers.
    fn send(&self, ask: CloudAsk, tell: Tell) -> Result<Box<dyn CloudRun>, CloudFault> {
        ask.check()?;
        let mut log = lock(&self.shared.log);
        if let Some(fault) = log.fail.take() {
            return Err(fault);
        }
        let id = log.open(ask, tell);
        drop(log);
        self.shared.tell(id, |log| log.take(id, Vec::new()));
        let shared = Arc::clone(&self.shared);
        Ok(Box::new(FakeCloudRun { shared, id }))
    }

    /// Checks `ask` by its own rule, records it without its key, waits while tests are held, then answers.
    fn test(&self, ask: CloudTest) -> Result<(), CloudFault> {
        ask.check()?;
        let mut log = lock(&self.shared.log);
        let id = log.calls.tests.len();
        let key_ok = log.key_ok(&ask.key);
        log.calls.tests.push(CloudTestLog::new(ask));
        self.shared.changed.notify_all();
        let mut log = self.shared.released(log);
        let answer = if key_ok {
            log.test_answers.pop_front().unwrap_or(Ok(()))
        } else {
            Err(CloudFault::Key)
        };
        log.calls.tests[id].answer = Some(answer);
        self.shared.changed.notify_all();
        answer
    }
}

#[cfg(test)]
mod tests;
