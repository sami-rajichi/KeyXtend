//! The job thread: it takes one job at a time from the actor, runs it, and hands the answer back. A panic in a job
//! answers that job as failed, so the actor never waits for good; a stop lets the running job end, runs no queued
//! one, and drops their passwords, wiped.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use super::{Ask, Done, Post, Runner};

/// The job thread's name, as a debugger shows it.
pub(crate) const JOB_THREAD: &str = "kx-vault-box";

/// The running job thread, its queue and its stop flag.
pub(crate) struct Jobs {
    tx: Option<Sender<Ask>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Jobs {
    /// Starts the thread over `runner`, which hands each answer to `reply`.
    ///
    /// # Errors
    /// The system's error when the thread did not start.
    pub(crate) fn spawn(
        runner: Runner,
        reply: impl Fn(Done) + Send + 'static,
    ) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let thread = thread::Builder::new()
            .name(JOB_THREAD.to_owned())
            .spawn(move || serve(runner, &rx, &reply, &flag))?;
        Ok(Self {
            tx: Some(tx),
            stop,
            thread: Some(thread),
        })
    }
}

/// Runs each job until the queue closes; once `stop` is set, the jobs left are dropped unrun.
pub(super) fn serve(
    mut runner: Runner,
    rx: &Receiver<Ask>,
    reply: &impl Fn(Done),
    stop: &AtomicBool,
) {
    while let Ok(Ask { round, job }) = rx.recv() {
        if stop.load(Ordering::Acquire) {
            return;
        }
        let failed = job.failed();
        let answer = catch_unwind(AssertUnwindSafe(|| runner.answer(job))).unwrap_or_else(|_| {
            tracing::warn!("a backup job panicked");
            failed
        });
        reply(Done { round, answer });
    }
}

impl Post for Jobs {
    fn post(&mut self, ask: Ask) {
        if let Some(tx) = &self.tx {
            // The thread ends only once this queue closes, so the send never fails.
            let _ = tx.send(ask);
        }
    }

    fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.tx = None;
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::warn!("the backup job thread panicked");
        }
    }
}

impl Drop for Jobs {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests;
