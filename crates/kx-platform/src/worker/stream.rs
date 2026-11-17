//! A streaming session (ADR-0042 decision 1), beside one ask and one reply: asks go out with a wait, and the worker
//! also sends notes on its own, each a checked frame told in order on the stream's thread. A watched id must hear a
//! note within its `Wait`, or the worker is ended; a frame its job cannot read ends it too.

mod flow;
mod watch;

pub use watch::Wait;

use super::{Frames, Launch, Lost, Spawner, Times, Worker, start};
use flow::{Flow, Pump};
use kx_ipc::IpcError;
use kx_module_api::{Clock, Mono};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, ThreadId};

/// What each reader thread's name adds to its stream's thread name.
const NOTES: &str = "notes";
/// The most frames of one worker that wait for the stream untaken; its reader then waits, so a flood stays in its pipe.
const IN_FLIGHT: usize = 2;
const _: () = assert!(IN_FLIGHT > 0, "a reader with no room would never wait");

/// A job whose worker also speaks on its own: each ask names the run it acts on, and some notes end their run's wait.
pub trait Streams: Frames {
    /// The run or ask id `ask` carries, which the worker's notes repeat.
    fn ask_id(ask: &Self::Ask) -> u32;

    /// True for a note after which its id needs no more watching: an answer, a run's end, or a fault that ends it.
    fn last(note: &Self::Reply) -> bool;

    /// True for a note after which a `Wait::Shift` takes its longer time; none to start.
    fn shift(_note: &Self::Reply) -> bool {
        false
    }

    /// Adds what `peak`, the job's most memory in MB, tells to a note that reports memory; none to start.
    fn stamp(_note: &mut Self::Reply, _peak: &mut dyn FnMut() -> Option<u32>) {}
}

/// A worker whose frames another thread may read, as a stream needs.
pub trait Split<F: Frames>: Worker<F> {
    /// Hands over the worker's frames as they come, for one reader thread; `reply` then hears `Closed`.
    fn notes(&mut self) -> Receiver<Result<F::Reply, IpcError>>;
}

/// What a stream tells, in order, on its own thread.
#[derive(Debug, PartialEq, Eq)]
pub enum Told<N> {
    /// A note the worker sent, checked by its job's frames.
    Note(N),
    /// The worker was ended, and every run on it with it: `Late` past a wait, `Gone` when it closed or sent a frame
    /// its job cannot read.
    Lost(Lost),
    /// The worker sat idle for its time and said goodbye, and every run on it with it.
    Idle,
}

/// Why an ask was not sent.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Unsent {
    /// The job's frames refuse it, so no worker saw it and none started.
    Refused(IpcError),
    /// No worker took it: its run's worker is gone, or none may start yet.
    Lost(Lost),
}

impl From<Lost> for Unsent {
    fn from(lost: Lost) -> Self {
        Self::Lost(lost)
    }
}

/// One call for the stream's thread, which sends its own answer back.
type Task<S, F> = Box<dyn FnOnce(&mut Flow<S, F>) + Send>;

/// What wakes the stream's thread.
enum Event<S: Spawner<F, Worker: Split<F>>, F: Streams> {
    /// A call from the owner.
    Task(Task<S, F>),
    /// A frame from worker `nth`, or why none was read, as its reader took it `at`.
    Heard {
        nth: u64,
        frame: Result<F::Reply, IpcError>,
        at: Mono,
        /// Dropped once the stream took the frame, which gives its reader room for another.
        taken: Sender<()>,
    },
    /// The stream was dropped.
    End,
}

/// A stream of job `F`; dropping it ends its thread, which says goodbye to the worker.
pub(crate) struct Stream<S: Spawner<F, Worker: Split<F>>, F: Streams> {
    /// `None` when the thread did not start; every ask is then `Gone`.
    events: Option<Sender<Event<S, F>>>,
    /// The stream's thread, where `tell` runs and no ask may wait.
    thread: Option<ThreadId>,
}

impl<S: Spawner<F, Worker: Split<F>>, F: Streams> Stream<S, F> {
    /// Runs a stream on thread `name` with no worker yet; the first fresh ask starts one as `launch` says. `tell`
    /// hears on that thread what the stream tells, a run's first notes perhaps before `begin` returns (so map a
    /// run's id inside `make`), and hands it on without waiting on this stream.
    pub(crate) fn open(
        name: &str,
        spawner: Arc<S>,
        clock: Arc<dyn Clock>,
        times: Times,
        launch: Launch,
        tell: impl FnMut(Told<F::Reply>) + Send + 'static,
    ) -> Self {
        let (events, inbox) = mpsc::channel();
        let (feed, stamp, reader) = (
            events.clone(),
            Arc::clone(&clock),
            format!("{name}-{NOTES}"),
        );
        let pump: Pump<F::Reply> = Box::new(move |nth, notes| {
            let (feed, stamp) = (feed.clone(), Arc::clone(&stamp));
            start(&reader, move || relay(nth, &notes, &feed, &*stamp))
        });
        let flow = Flow::new(spawner, clock, times, launch, Box::new(tell), pump);
        let started = thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || serve(flow, &inbox));
        let started = started
            .inspect_err(|e| tracing::warn!(thread = name, error = %e, "a stream did not start"))
            .ok();
        let thread = started.as_ref().map(|t| t.thread().id());
        Self {
            events: started.map(|_| events),
            thread,
        }
    }

    /// Sends the ask `make` builds from a fresh id, the worker started first if need be, and watches that id as
    /// `wait` says; returns the id.
    ///
    /// # Errors
    /// `Refused` for an ask the frames refuse or one built with another id, `Down` while the backoff runs or when no
    /// worker starts, `Gone` without the stream's thread or from it.
    pub(crate) fn begin(
        &self,
        make: impl FnOnce(u32) -> F::Ask + Send + 'static,
        wait: Wait,
    ) -> Result<u32, Unsent> {
        let begun = self.run(move |flow| flow.begin(make, wait));
        begun.unwrap_or(Err(Lost::Gone.into()))
    }

    /// Sends `ask` on a run the live worker took, and watches its id as `wait` says.
    ///
    /// # Errors
    /// `Refused` for an ask the frames refuse; `Gone` when its run's worker ended, as no worker starts for it.
    pub(crate) fn ask(&self, ask: F::Ask, wait: Wait) -> Result<(), Unsent> {
        let sent = self.run(move |flow| flow.ask(ask, wait));
        sent.unwrap_or(Err(Lost::Gone.into()))
    }

    /// Says goodbye to the worker now and drops every watch; the next fresh ask starts another.
    #[cfg(test)]
    pub(crate) fn stop(&self) {
        let _ = self.run(Flow::stop);
    }

    /// Says goodbye to the worker now, and starts the next as `launch` and `times` say; the backoff stays.
    pub(crate) fn relaunch(&self, launch: Launch, times: Times) {
        let _ = self.run(move |flow| flow.relaunch(launch, times));
    }

    /// Wakes the stream's thread, which then checks its waits and its idle time against its clock.
    #[cfg(test)]
    pub(crate) fn poke(&self) {
        let _ = self.run(|_| ());
    }

    /// Runs `call` on the stream's thread and waits for its answer; `None` without the thread, or on it.
    fn run<T: Send + 'static>(
        &self,
        call: impl FnOnce(&mut Flow<S, F>) -> T + Send + 'static,
    ) -> Option<T> {
        if self.thread == Some(thread::current().id()) {
            tracing::warn!(job = F::JOB, "a stream was asked from its own thread");
            return None;
        }
        let events = self.events.as_ref()?;
        let (answer, heard) = mpsc::channel();
        let task: Task<S, F> = Box::new(move |flow| drop(answer.send(call(flow))));
        events.send(Event::Task(task)).ok()?;
        heard.recv().ok()
    }
}

impl<S: Spawner<F, Worker: Split<F>>, F: Streams> Drop for Stream<S, F> {
    /// Tells the thread to end without joining it or waiting for the goodbye, as `Session` does; at app exit the
    /// Windows job's kill-on-close ends the worker.
    fn drop(&mut self) {
        if let Some(events) = &self.events {
            let _ = events.send(Event::End);
        }
    }
}

/// The stream's thread: it runs each call and takes each frame in turn, and checks its waits only when nothing came
/// within them, so a frame that waits is taken first.
fn serve<S: Spawner<F, Worker: Split<F>>, F: Streams>(
    mut flow: Flow<S, F>,
    inbox: &Receiver<Event<S, F>>,
) {
    loop {
        let next = match flow.wake_in() {
            Some(wait) => inbox.recv_timeout(wait),
            None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match next {
            Ok(Event::Task(task)) => task(&mut flow),
            Ok(Event::Heard {
                nth,
                frame,
                at,
                taken,
            }) => {
                flow.heard(nth, frame, at);
                drop(taken);
            }
            Ok(Event::End) | Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => flow.tick(),
        }
    }
}

/// A reader thread: it stamps each frame of worker `nth` and hands it to the stream, with `IN_FLIGHT` untaken at
/// most, until one fails or the stream is gone.
fn relay<S: Spawner<F, Worker: Split<F>>, F: Streams>(
    nth: u64,
    notes: &Receiver<Result<F::Reply, IpcError>>,
    feed: &Sender<Event<S, F>>,
    clock: &dyn Clock,
) {
    let mut untaken = VecDeque::with_capacity(IN_FLIGHT);
    loop {
        if untaken.len() >= IN_FLIGHT {
            // Ends once the stream took the oldest frame, or dropped it with the stream.
            let _ = untaken
                .pop_front()
                .map(|oldest: Receiver<()>| oldest.recv());
        }
        let frame = notes.recv().unwrap_or(Err(IpcError::Closed));
        let (last, at, (taken, room)) = (frame.is_err(), clock.now(), mpsc::channel());
        let heard = Event::Heard {
            nth,
            frame,
            at,
            taken,
        };
        if feed.send(heard).is_err() || last {
            return;
        }
        untaken.push_back(room);
    }
}

#[cfg(test)]
mod tests;
