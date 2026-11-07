//! A port's session thread: it owns the host, runs each call in turn, stops an idle worker, and ends with the session.

use super::{Frames, Host, Spawner};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;

/// One call for the session thread, which sends its own answer back.
type Task<S, F> = Box<dyn FnOnce(&mut Host<S, F>) + Send>;

/// A session; dropping it ends its thread, which stops the worker.
pub(crate) struct Session<S: Spawner<F>, F: Frames> {
    /// `None` when the thread did not start; every call then has no answer.
    tasks: Option<Sender<Task<S, F>>>,
}

impl<S: Spawner<F>, F: Frames> Session<S, F> {
    /// Runs `host` on a thread named `name`; a thread that does not start leaves the session without one.
    pub(crate) fn open(name: &str, host: Host<S, F>) -> Self {
        let (tasks, inbox) = mpsc::channel();
        let started = thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || serve(host, &inbox));
        let tasks = started
            .inspect_err(
                |e| tracing::warn!(thread = name, error = %e, "a session thread did not start"),
            )
            .ok()
            .map(|_| tasks);
        Self { tasks }
    }

    /// Runs `call` on the session thread and waits for its answer; `None` without the thread.
    pub(crate) fn run<T: Send + 'static>(
        &self,
        call: impl FnOnce(&mut Host<S, F>) -> T + Send + 'static,
    ) -> Option<T> {
        let tasks = self.tasks.as_ref()?;
        let (answer, heard) = mpsc::channel();
        // The call's inputs move on to the session thread, so they are never copied here.
        let task: Task<S, F> = Box::new(move |host| drop(answer.send(call(host))));
        tasks.send(task).ok()?;
        heard.recv().ok()
    }
}

/// The session thread: it runs each call, stops an idle worker, and ends with the session.
fn serve<S: Spawner<F>, F: Frames>(mut host: Host<S, F>, inbox: &Receiver<Task<S, F>>) {
    loop {
        let next = match host.wake_in() {
            Some(wait) => inbox.recv_timeout(wait),
            None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match next {
            Ok(task) => task(&mut host),
            Err(RecvTimeoutError::Timeout) => host.idle(),
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}
