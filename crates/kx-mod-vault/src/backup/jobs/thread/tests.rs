//! The job thread: every box call runs there, the stop ends the session, and a stop runs no queued job.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use kx_module_api::ports::{BoxError, BoxPart, BoxRun, Boxes, KeepassImport, Opened, Padlock};
use kx_module_api::{Redacted, Secret};
use kx_platform_fake::BoxCall;

use super::super::tests::{CFG, PASSWORD, WITHIN, fakes, secret};
use super::super::{Answer, Ask, Done, Job, Post, Runner};
use super::{JOB_THREAD, Jobs, serve};

/// A session that notes the thread of each call, and sets `stop` as a call runs when given.
struct Spy {
    inner: Box<dyn BoxRun>,
    seen: Arc<Mutex<Vec<Option<String>>>>,
    stop: Option<Arc<AtomicBool>>,
}

impl Spy {
    fn note(&self) {
        let name = std::thread::current().name().map(str::to_owned);
        self.seen.lock().unwrap().push(name);
        if let Some(stop) = &self.stop {
            stop.store(true, Ordering::Release);
        }
    }
}

impl BoxRun for Spy {
    fn padlock(&mut self, password: &Secret, work: u8) -> Result<Padlock, BoxError> {
        self.note();
        self.inner.padlock(password, work)
    }

    fn seal(
        &mut self,
        padlock: &Padlock,
        parts: &[BoxPart],
    ) -> Result<Redacted<Vec<u8>>, BoxError> {
        self.note();
        self.inner.seal(padlock, parts)
    }

    fn open(&mut self, bytes: &[u8], password: &Secret) -> Result<Opened, BoxError> {
        self.note();
        self.inner.open(bytes, password)
    }

    fn check(&mut self, padlock: &Padlock, password: &Secret) -> Result<bool, BoxError> {
        self.note();
        self.inner.check(padlock, password)
    }

    fn keepass(&mut self, bytes: &[u8], password: &Secret) -> Result<KeepassImport, BoxError> {
        self.note();
        self.inner.keepass(bytes, password)
    }
}

/// A padlock job of round `round`.
fn padlock_job(round: u32) -> Ask {
    let password = secret(PASSWORD);
    Ask {
        round,
        job: Job::Padlock { password, work: 18 },
    }
}

#[test]
fn every_box_call_runs_on_the_job_thread_and_stopping_ends_the_session() {
    let (boxes, places) = fakes();
    let seen = Arc::default();
    let spy = Spy {
        inner: boxes.start(CFG),
        seen: Arc::clone(&seen),
        stop: None,
    };
    let (to_test, answers) = mpsc::channel();
    let runner = Runner::new(Box::new(spy), Arc::new(places));
    let mut jobs = Jobs::spawn(runner, move |done| drop(to_test.send(done))).unwrap();
    jobs.post(padlock_job(7));
    let done = answers.recv_timeout(WITHIN).unwrap();
    let made = matches!(
        done,
        Done {
            round: 7,
            answer: Answer::Padlock(Ok(_))
        }
    );
    assert!(made, "{done:?}");
    assert_eq!(*seen.lock().unwrap(), [Some(JOB_THREAD.to_owned())]);
    assert_ne!(std::thread::current().name(), Some(JOB_THREAD));
    jobs.stop();
    assert!(!boxes.running(), "the session ends with the thread");
}

#[test]
fn a_stop_while_a_job_runs_runs_no_queued_job() {
    let (boxes, places) = fakes();
    let stop = Arc::new(AtomicBool::new(false));
    let spy = Spy {
        inner: boxes.start(CFG),
        seen: Arc::default(),
        stop: Some(Arc::clone(&stop)),
    };
    let (tx, rx) = mpsc::channel();
    for round in [1, 2] {
        tx.send(padlock_job(round)).unwrap();
    }
    drop(tx);
    let answered = Mutex::new(Vec::new());
    let runner = Runner::new(Box::new(spy), Arc::new(places));
    serve(
        runner,
        &rx,
        &|done: Done| answered.lock().unwrap().push(done.round),
        &stop,
    );
    assert_eq!(*answered.lock().unwrap(), [1], "the running job answers");
    assert_eq!(
        boxes.calls(BoxCall::Padlock),
        1,
        "the queued one never runs"
    );
}
