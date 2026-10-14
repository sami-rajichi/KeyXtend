//! The Win32 side of the pointer hook: its thread with both low-level hooks, the message loop, and `release_now`, with
//! which the panic hook lets go and keeps the run. One run lives per process.

mod os;
mod procs;
mod release;

use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use kx_module_api::ports::{HookConfig, HookError, HookRun, HookStats, PointerEngine, PointerHook};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_APP, WM_QUIT, WM_TIMER};

use super::Source;
use super::host::Host;
use crate::QpcClock;
use os::{Ctl, WinOs, kill_timer, own_queue, post, raise_priority, set_timer, unhook_all};
use release::{LET_GO, forget_asks, let_go};

pub use release::release_now;

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
/// The hook thread's name.
const THREAD: &str = "kx-pointer-hook";

/// The live hook thread's id, `NONE` or `STARTING`; `release_now` finds the thread here.
static LIVE: AtomicU32 = AtomicU32::new(NONE);

thread_local! {
    /// The host, on the hook thread only.
    static HOST: RefCell<Option<Host<WinOs>>> = const { RefCell::new(None) };
}

/// The Windows pointer hook: `WH_MOUSE_LL` and `WH_KEYBOARD_LL` on a thread of their own, one run per process.
/// Dropping a run blocks until that thread has let go, sent that input and removed the hooks, or for `STOP_WAIT`
/// at most.
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
        forget_asks();
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
    answer: &Receiver<Result<(), HookError>>,
    ctl: Arc<Ctl>,
    ended: Receiver<()>,
    join: JoinHandle<()>,
) -> Result<Box<dyn HookRun>, HookError> {
    match answer.recv_timeout(START_WAIT) {
        Ok(Ok(())) => Ok(Box::new(WinRun {
            ctl,
            ended: Mutex::new(ended),
            join: Some(join),
        })),
        Ok(Err(error)) => {
            let _ = join.join();
            Err(error)
        }
        Err(_) => {
            ask_stop(&ctl, &LIVE, &ended);
            Err(HookError::Thread)
        }
    }
}

/// Asks the run's thread to stop, and wakes it with `WM_QUIT` unless it ended, so a thread that answered late stops
/// too.
fn ask_stop(ctl: &Ctl, live: &AtomicU32, ended: &Receiver<()>) {
    ctl.stop.store(true, Ordering::SeqCst);
    if !matches!(ended.try_recv(), Err(TryRecvError::Disconnected)) {
        let _ = post_live(live, ctl.tid.load(Ordering::SeqCst), WM_QUIT);
    }
}

/// Posts `msg` to the run's thread `tid` only while `live` holds that id, so a thread that ended gets nothing.
fn post_live(live: &AtomicU32, tid: u32, msg: u32) -> bool {
    tid != NONE && live.load(Ordering::SeqCst) == tid && post(tid, msg)
}

/// A live run; see `Drop`.
struct WinRun {
    ctl: Arc<Ctl>,
    /// Hears the thread end; in a `Mutex` only so the run is `Sync`.
    ended: Mutex<Receiver<()>>,
    join: Option<JoinHandle<()>>,
}

impl HookRun for WinRun {
    fn wake(&self) {
        let _ = post_live(&LIVE, self.ctl.tid.load(Ordering::SeqCst), WAKE);
    }

    fn stats(&self) -> HookStats {
        self.ctl.stats.snapshot()
    }
}

impl Drop for WinRun {
    /// Blocks until the thread has let go, sent that input and removed the hooks. After `STOP_WAIT` it removes the
    /// hooks itself, warns once and leaves the thread, so a module's stop never hangs.
    fn drop(&mut self) {
        let ended = self.ended.get_mut().unwrap_or_else(PoisonError::into_inner);
        ask_stop(&self.ctl, &LIVE, ended);
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
    ready: Sender<Result<(), HookError>>,
    /// Dropped as the thread ends, which the run's drop hears.
    _done: Sender<()>,
}

impl Job {
    /// The hook thread: installs both hooks and answers, loops until stopped, then lets go and removes them.
    fn run(self) {
        let _ending = Ending(Arc::clone(&self.ctl));
        raise_priority();
        let tid = own_queue();
        self.ctl.tid.store(tid, Ordering::SeqCst);
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
        if health == 0 {
            tracing::warn!("the pointer hook's health check could not start");
        }
        LIVE.store(tid, Ordering::SeqCst);
        let _ = ready.send(Ok(()));
        pump(&ctl, health);
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

/// Runs this thread's queue until our own stop: the run's drop or a start that gave up. Another program's `WM_QUIT` is
/// ignored.
#[allow(unsafe_code, reason = "The message loop of this thread's own queue.")]
fn pump(ctl: &Ctl, health: usize) {
    let mut msg = MSG::default();
    while !stopping(ctl) {
        // SAFETY: the loop of this thread's own queue; `msg` is a local it fills.
        let got = unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0;
        if got < 0 || stopping(ctl) {
            break;
        }
        match msg.message {
            WM_TIMER if msg.wParam.0 == health => with_host(Host::check),
            WAKE => with_host(Host::turn),
            WM_TIMER => with_host(Host::fired),
            LET_GO => let_go(),
            _ => None,
        };
    }
}

/// True once our stop was asked for.
fn stopping(ctl: &Ctl) -> bool {
    ctl.stop.load(Ordering::SeqCst)
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

#[cfg(test)]
mod tests;
