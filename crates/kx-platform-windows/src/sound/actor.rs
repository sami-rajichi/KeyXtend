//! The speaker thread and its bounded queue: `Speaker` calls never wait, and the thread owns the engine.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use kx_module_api::Clock;
use kx_platform::ui::{Clip, ClipId, Speaker, SpeakerConfig};

use super::{Mixer, Os};
use crate::lock;

/// The speaker thread's name.
const THREAD: &str = "kx-speaker";
/// The most asks that wait for the thread; a play past it is dropped, since a late sound is wrong.
pub(crate) const QUEUE: usize = 32;
/// How long dropping the speaker waits for its thread before it leaves the thread to end alone.
const STOP_WAIT: Duration = Duration::from_secs(1);

/// The clips and settings the thread prepares next.
type Prepared = (Vec<Clip>, SpeakerConfig);

/// A request to the speaker thread.
enum Ask {
    /// New clips wait in `Shared`, so a full queue never loses them.
    Prepare,
    Play(ClipId, f32),
}

/// What the speaker and its thread share.
#[derive(Debug, Default)]
struct Shared {
    /// Set as the speaker drops: the thread ends without serving the queue.
    stop: AtomicBool,
    /// The latest clips the thread has not prepared yet.
    prepared: Mutex<Option<Prepared>>,
}

/// The running thread.
#[derive(Debug)]
struct Thread {
    asks: SyncSender<Ask>,
    /// Hears the thread end; in a `Mutex` only so the speaker is `Sync`.
    ended: Mutex<Receiver<()>>,
    join: JoinHandle<()>,
}

/// The speaker thread and its queue. Dropping it stops the thread, which drops the voices then the engine, and waits
/// `STOP_WAIT` at most.
#[derive(Debug)]
pub(crate) struct Actor {
    shared: Arc<Shared>,
    /// `None` if the thread did not start; then nothing plays.
    thread: Option<Thread>,
    /// How long the drop waits for the thread.
    wait: Duration,
}

impl Actor {
    /// Starts the thread, which makes its system with `make` and reads the time from `clock`; without it nothing plays.
    pub(crate) fn spawn<O: Os + 'static>(
        make: impl FnOnce() -> O + Send + 'static,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let shared = Arc::new(Shared::default());
        let (theirs, (asks, queue), (done, ended)) = (
            Arc::clone(&shared),
            mpsc::sync_channel(QUEUE),
            mpsc::channel(),
        );
        let body = move || {
            // Dropped last, once the voices and the engine are gone, so the speaker's drop hears a full stop.
            let _done: mpsc::Sender<()> = done;
            run(make(), &queue, &theirs, clock.as_ref());
        };
        let spawned = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(body)
            .inspect_err(
                |e| tracing::warn!(error = %e, "no sound: the speaker thread did not start"),
            );
        let thread = spawned.ok().map(|join| Thread {
            asks,
            ended: Mutex::new(ended),
            join,
        });
        Self {
            shared,
            thread,
            wait: STOP_WAIT,
        }
    }

    /// Sets how long the drop waits for the thread.
    #[cfg(test)]
    pub(crate) fn stop_wait(&mut self, wait: Duration) {
        self.wait = wait;
    }

    /// Queues `ask` if there is room; a full queue or an ended thread drops it, and the call returns at once.
    fn send(&self, ask: Ask) {
        if let Some(thread) = &self.thread {
            let _ = thread.asks.try_send(ask);
        }
    }
}

impl Speaker for Actor {
    fn prepare(&self, clips: Vec<Clip>, config: SpeakerConfig) {
        *lock(&self.shared.prepared) = Some((clips, config));
        self.send(Ask::Prepare);
    }

    fn play(&self, id: ClipId, gain: f32) {
        self.send(Ask::Play(id, gain));
    }
}

impl Drop for Actor {
    /// Stops the thread and waits for it to drop the voices and the engine. After `wait` it warns and leaves the
    /// thread, so closing never hangs.
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::SeqCst);
        let Some(Thread { asks, ended, join }) = self.thread.take() else {
            return;
        };
        drop(asks);
        let ended = ended.into_inner().unwrap_or_else(PoisonError::into_inner);
        if matches!(
            ended.recv_timeout(self.wait),
            Err(RecvTimeoutError::Timeout)
        ) {
            let wait_ms = self.wait.as_millis();
            tracing::warn!(wait_ms, "the speaker thread did not stop in time");
            return;
        }
        let _ = join.join();
    }
}

/// The thread: it prepares the latest clips, plays and stops the idle engine until the speaker stops, then drops the
/// voices and the engine.
fn run<O: Os>(os: O, queue: &Receiver<Ask>, shared: &Shared, clock: &dyn Clock) {
    let mut mixer = Mixer::new(os);
    loop {
        let ask = match mixer.wait(clock.now()) {
            Some(wait) => queue.recv_timeout(wait),
            None => queue.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        let gone = matches!(ask, Err(RecvTimeoutError::Disconnected));
        if gone || shared.stop.load(Ordering::SeqCst) {
            return;
        }
        let prepared = lock(&shared.prepared).take();
        if let Some((clips, config)) = prepared {
            mixer.prepare(clips, config);
        }
        if let Ok(Ask::Play(id, gain)) = ask {
            mixer.play(id, gain, clock.now());
        }
        mixer.idle(clock.now());
    }
}
