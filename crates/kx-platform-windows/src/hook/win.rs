//! The Win32 side of the pointer hook: its thread with both low-level hooks, the message loop, and `release_now` for
//! the panic hook. One run lives per process.

mod os;
mod procs;

use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use kx_module_api::ports::{HookConfig, HookError, HookRun, HookStats, PointerEngine, PointerHook};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_APP, WM_QUIT, WM_TIMER};

use super::Source;
use super::host::Host;
use crate::QpcClock;
use os::{
    Ctl, WinOs, kill_timer, own_queue, post, raise_priority, set_timer, this_thread, unhook_all,
};

/// Asks the hook thread for a turn.
const WAKE: u32 = WM_APP + 1;
/// `LIVE` when no hook thread runs.
const NONE: u32 = 0;
/// `LIVE` while a hook starts; thread ids are multiples of four, so this is never one.
const STARTING: u32 = u32::MAX;
/// How long `start` waits for the hook thread's answer.
const START_WAIT: Duration = Duration::from_secs(2);
/// How long dropping a run waits for its thread to let go and stop.
const STOP_WAIT: Duration = Duration::from_secs(1);
/// How long `release_now` waits for the thread to let go.
const RELEASE_WAIT: Duration = Duration::from_millis(200);
/// How often `release_now` looks while it waits.
const RELEASE_POLL: Duration = Duration::from_millis(1);
/// The hook thread's name.
const THREAD: &str = "kx-pointer-hook";

/// The live hook thread's id, `NONE` or `STARTING`; `release_now` finds the thread here.
static LIVE: AtomicU32 = AtomicU32::new(NONE);
/// The thread `release_now` asked to stop.
static RELEASE: AtomicU32 = AtomicU32::new(NONE);

thread_local! {
    /// The host, on the hook thread only.
    static HOST: RefCell<Option<Host<WinOs>>> = const { RefCell::new(None) };
}

/// The Windows pointer hook: `WH_MOUSE_LL` and `WH_KEYBOARD_LL` on a thread of their own, one run per process.
/// Dropping a run blocks until that thread has let go, sent that input and removed the hooks, or for `STOP_WAIT` at most.
#[derive(Clone, Copy, Debug)]
pub struct WinPointerHook {
    clock: QpcClock,
    source: Source,
}

impl WinPointerHook {
    /// A hook whose engine acts on `source`'s input, timed by the platform's `clock`; only the hand's without the
    /// `sim-input` feature.
    #[must_use]
    pub fn new(clock: QpcClock, source: Source) -> Self {
        Self { clock, source }
    }
}

impl PointerHook for WinPointerHook {
    fn start(
        &self,
        engine: Box<dyn PointerEngine>,
        config: HookConfig,
    ) -> Result<Box<dyn HookRun>, HookError> {
        if LIVE
            .compare_exchange(NONE, STARTING, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(HookError::Busy);
        }
        RELEASE.store(NONE, Ordering::SeqCst);
        let ctl = Arc::new(Ctl::default());
        let (ready, answer) = mpsc::channel();
        let (done, ended) = mpsc::channel();
        let job = Job {
            engine,
            config,
            source: self.source,
            clock: self.clock,
            ctl: Arc::clone(&ctl),
            ready,
            _done: done,
        };
        let spawned = thread::Builder::new()
            .name(THREAD.to_owned())
            .spawn(move || job.run());
        let Ok(join) = spawned else {
            LIVE.store(NONE, Ordering::SeqCst);
            return Err(HookError::Thread);
        };
        wait_ready(&answer, ctl, ended, join)
    }
}

/// The started run once the thread answers; a refusal waits for the thread to end, and silence asks it to stop.
fn wait_ready(
    answer: &Receiver<Result<u32, HookError>>,
    ctl: Arc<Ctl>,
    ended: Receiver<()>,
    join: JoinHandle<()>,
) -> Result<Box<dyn HookRun>, HookError> {
    match answer.recv_timeout(START_WAIT) {
        Ok(Ok(tid)) => Ok(Box::new(WinRun {
            ctl,
            tid,
            ended: Mutex::new(ended),
            join: Some(join),
        })),
        Ok(Err(error)) => {
            let _ = join.join();
            Err(error)
        }
        Err(_) => {
            ctl.stop.store(true, Ordering::SeqCst);
            Err(HookError::Thread)
        }
    }
}

/// A live run; see `Drop`.
struct WinRun {
    ctl: Arc<Ctl>,
    tid: u32,
    /// Hears the thread end; in a `Mutex` only so the run is `Sync`.
    ended: Mutex<Receiver<()>>,
    join: Option<JoinHandle<()>>,
}

impl HookRun for WinRun {
    fn wake(&self) {
        let _ = post(self.tid, WAKE);
    }

    fn stats(&self) -> HookStats {
        self.ctl.stats.snapshot()
    }
}

impl Drop for WinRun {
    /// Blocks until the thread has let go, sent that input and removed the hooks. After `STOP_WAIT` it removes the
    /// hooks itself, warns once and leaves the thread, so a module's stop never hangs.
    fn drop(&mut self) {
        self.ctl.stop.store(true, Ordering::SeqCst);
        let _ = post(self.tid, WM_QUIT);
        let ended = self.ended.get_mut().unwrap_or_else(PoisonError::into_inner);
        if matches!(
            ended.recv_timeout(STOP_WAIT),
            Err(RecvTimeoutError::Timeout)
        ) {
            unhook_all(&self.ctl);
            let wait_ms = STOP_WAIT.as_millis();
            tracing::warn!(wait_ms, "the pointer hook thread did not stop in time");
            return;
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// What the hook thread takes with it.
struct Job {
    engine: Box<dyn PointerEngine>,
    config: HookConfig,
    source: Source,
    clock: QpcClock,
    ctl: Arc<Ctl>,
    ready: Sender<Result<u32, HookError>>,
    /// Dropped as the thread ends, which the run's drop hears.
    _done: Sender<()>,
}

impl Job {
    /// The hook thread: installs both hooks and answers, loops until stopped, then lets go and removes them.
    fn run(self) {
        let _ending = Ending(Arc::clone(&self.ctl));
        raise_priority();
        let tid = own_queue();
        let Self {
            engine,
            config,
            source,
            clock,
            ctl,
            ready,
            _done,
        } = self;
        let os = WinOs::new(tid, clock, Arc::clone(&ctl));
        let host = Host::new(os, engine, source, &config, Arc::clone(&ctl.stats));
        if let Err(error) = begin(host) {
            let _ = ready.send(Err(error));
            return;
        }
        let health = set_timer(config.health_ms);
        LIVE.store(tid, Ordering::SeqCst);
        let _ = ready.send(Ok(tid));
        pump(&ctl, tid, health);
        kill_timer(health);
        let _ = with_host(Host::stop);
        drop_host();
    }
}

/// Puts `host` in this thread's slot and installs both hooks; after a refusal the host is dropped again.
fn begin(host: Host<WinOs>) -> Result<(), HookError> {
    let _ = HOST.try_with(|h| h.try_borrow_mut().map(|mut slot| *slot = Some(host)));
    let error = match with_host(Host::begin) {
        Some(Ok(())) => return Ok(()),
        Some(Err(code)) => HookError::Refused(code),
        None => HookError::Thread,
    };
    drop_host();
    Err(error)
}

/// As the hook thread ends, even by a panic, removes any hook still in and clears `LIVE`.
struct Ending(Arc<Ctl>);

impl Drop for Ending {
    fn drop(&mut self) {
        unhook_all(&self.0);
        LIVE.store(NONE, Ordering::SeqCst);
    }
}

/// Runs this thread's queue until our own stop: the run's drop, a start that gave up, or `release_now`. Another
/// program's `WM_QUIT` is ignored.
#[allow(unsafe_code, reason = "The message loop of this thread's own queue.")]
fn pump(ctl: &Ctl, tid: u32, health: usize) {
    let mut msg = MSG::default();
    while !stopping(ctl, tid) {
        // SAFETY: the loop of this thread's own queue; `msg` is a local it fills.
        let got = unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0;
        if got < 0 || stopping(ctl, tid) {
            break;
        }
        match msg.message {
            WM_TIMER if msg.wParam.0 == health => with_host(Host::check),
            WAKE | WM_TIMER => with_host(Host::turn),
            _ => None,
        };
    }
}

/// True once our stop was asked for this thread.
fn stopping(ctl: &Ctl, tid: u32) -> bool {
    ctl.stop.load(Ordering::SeqCst) || RELEASE.load(Ordering::SeqCst) == tid
}

/// Asks a live hook thread to let go of everything and stop, and waits at most `RELEASE_WAIT`; for the app's panic
/// hook. It never panics, and does nothing with no live hook or on the hook thread itself.
pub fn release_now() {
    let tid = LIVE.load(Ordering::SeqCst);
    if tid == NONE || tid == STARTING || tid == this_thread() {
        return;
    }
    RELEASE.store(tid, Ordering::SeqCst);
    let _ = post(tid, WM_QUIT);
    let start = Instant::now();
    while LIVE.load(Ordering::SeqCst) == tid && start.elapsed() < RELEASE_WAIT {
        thread::sleep(RELEASE_POLL);
    }
}

/// Runs `f` on this thread's host; `None` with no host, or while it is busy.
fn with_host<T>(f: impl FnOnce(&mut Host<WinOs>) -> T) -> Option<T> {
    HOST.try_with(|h| h.try_borrow_mut().ok()?.as_mut().map(f))
        .ok()
        .flatten()
}

/// Drops this thread's host, and the engine with it, on this thread.
fn drop_host() {
    let _ = HOST.try_with(|h| h.try_borrow_mut().map(|mut host| host.take()));
}
