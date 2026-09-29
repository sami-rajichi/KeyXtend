//! The hook thread's state: the hold engine, the input queued behind ours, and the hold timer.
//!
//! Input the engine lets through never overtakes our own queued input: it waits behind it until that comes back.
#![cfg(windows)]

use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use spike_core::clock::now_us;
use spike_core::hold::{Act, Engine, Event, Mode, Pt};
use spike_core::inject::TAG;
use windows::Win32::Foundation::{LPARAM, RECT, WPARAM};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::WindowsAndMessaging::{
    KBDLLHOOKSTRUCT, KillTimer, LLKHF_INJECTED, LLMHF_INJECTED, MSLLHOOKSTRUCT, PostThreadMessageW,
    SetTimer, WM_APP,
};

use crate::assist::Setup;
use crate::hookio::{self, Source};
use crate::stats::US_PER_MS_INT;

/// Wakes the hook thread: commands or injections are waiting.
pub const WAKE: u32 = WM_APP + 1;
/// One extra ms, so the hold timer never fires before its deadline.
const ROUND_UP_MS: i64 = 1;
/// Most hook timings kept; later calls still run but are not timed.
const HOOK_SAMPLES: usize = 1 << 16;

/// A request to the hook thread.
pub enum Cmd {
    /// Switch the engine's mode.
    Mode(Mode),
    /// Hold a modifier key down until the next left click ends.
    Latch(u16),
    /// Let go of everything and end the thread.
    Stop,
}

/// A request, and where to answer once it is applied.
pub type Call = (Cmd, Sender<()>);

/// What the hook thread measured.
#[derive(Debug, Default, Clone)]
pub struct Report {
    /// Time spent in each hook call, in µs.
    pub hook_us: Vec<i64>,
    /// The longest an event waited before our hook ran, in ms.
    pub max_wait_ms: u32,
    /// Events the engine was given.
    pub seen: usize,
    /// Injection errors.
    pub errors: Vec<String>,
}

/// Values kx-gates reads while the engine runs.
#[derive(Debug, Default, Clone, Copy)]
pub struct Live {
    /// The last pointer point outside our own windows.
    pub last_outside: Option<Pt>,
    /// Events the engine was given so far.
    pub seen: usize,
}

/// State owned by the hook thread; the hook callbacks reach it through `HOST`.
pub struct Host {
    engine: Engine,
    source: Source,
    own: Vec<RECT>,
    tid: u32,
    live: Arc<Mutex<Live>>,
    /// Input to inject once the hook has returned.
    acts: Vec<Act>,
    timer: usize,
    report: Report,
    /// When our last mouse input went out; later input waits until it comes back through the hook.
    echo_since: Option<i64>,
    /// How long to wait for that, in µs.
    echo_wait_us: i64,
}

/// Milliseconds from `now` until `due` (both µs), rounded up so the timer is never early.
fn timer_ms(due: i64, now: i64) -> u32 {
    let ms = due.saturating_sub(now).max(0) / US_PER_MS_INT;
    u32::try_from(ms.saturating_add(ROUND_UP_MS)).unwrap_or(u32::MAX)
}

impl Host {
    /// A host for `setup`, on the thread `tid`.
    pub fn new(setup: Setup, tid: u32, live: Arc<Mutex<Live>>) -> Self {
        let us = |ms: u64| {
            i64::try_from(ms)
                .ok()
                .and_then(|ms| ms.checked_mul(US_PER_MS_INT))
                .unwrap_or(i64::MAX)
        };
        Self {
            engine: Engine::new(us(setup.hold_ms), setup.still_px),
            source: setup.source,
            own: setup.own,
            tid,
            live,
            acts: Vec::new(),
            timer: 0,
            report: Report {
                hook_us: Vec::with_capacity(HOOK_SAMPLES),
                ..Report::default()
            },
            echo_since: None,
            echo_wait_us: us(setup.reply_ms),
        }
    }

    #[allow(unsafe_code, reason = "Posting to our own thread's queue.")]
    fn wake(&self) {
        // SAFETY: posting to our own thread's queue.
        let _ = unsafe { PostThreadMessageW(self.tid, WAKE, WPARAM(0), LPARAM(0)) };
    }

    /// Our mouse input went out at `at` (µs); later input waits behind it.
    pub fn sent(&mut self, at: i64) {
        self.echo_since = Some(at);
    }

    /// Our injection failed with `error`, so no echo is coming.
    pub fn failed(&mut self, error: String) {
        self.echo_since = None;
        self.report.errors.push(error);
    }

    /// Lets go of everything the engine holds; returns the input that does it.
    pub fn reset(&mut self) -> Vec<Act> {
        self.engine.reset()
    }

    /// What the thread measured.
    pub fn into_report(self) -> Report {
        self.report
    }

    /// True while our input is queued or not back yet; gives up waiting after the echo wait.
    fn behind(&mut self) -> bool {
        let now = now_us();
        if self.echo_since.is_some_and(|t| now - t > self.echo_wait_us) {
            self.echo_since = None;
            let lost = "our input never came back through the hook".to_string();
            self.report.errors.push(lost);
        }
        !self.acts.is_empty() || self.echo_since.is_some()
    }

    /// Gives the engine `event` at `at` (µs); input it lets through goes behind ours, in order.
    fn queue(&mut self, event: Event, at: i64) -> bool {
        let (behind, before, queued) = (self.behind(), self.engine.deadline(), self.acts.len());
        let step = self.engine.on(event, at);
        let mut swallow = step.swallow;
        if behind
            && !swallow
            && let Some(act) = hookio::act_of(event)
        {
            self.acts.push(act);
            swallow = true;
        }
        self.acts.extend(step.acts);
        if self.acts.len() > queued || self.engine.deadline() != before {
            self.wake();
        }
        swallow
    }

    /// When the event happened, in µs: its Windows tick time, which also shows how long it waited for us.
    #[allow(unsafe_code, reason = "Plain query.")]
    fn event_time(&mut self, tick: u32, now: i64) -> i64 {
        // SAFETY: plain query.
        let waited = unsafe { GetTickCount() }.wrapping_sub(tick);
        self.report.max_wait_ms = self.report.max_wait_ms.max(waited);
        now - i64::from(waited) * US_PER_MS_INT
    }

    /// Counts an event and remembers the last point outside our windows; never waits for the lock.
    fn note(&mut self, pt: Option<(Pt, bool)>, took_us: i64) {
        self.report.seen += 1;
        if self.report.hook_us.len() < HOOK_SAMPLES {
            self.report.hook_us.push(took_us);
        }
        if let Ok(mut live) = self.live.try_lock() {
            live.seen = self.report.seen;
            if let Some((p, false)) = pt {
                live.last_outside = Some(p);
            }
        }
    }

    /// Handles a mouse event `msg`; true to swallow it.
    pub fn on_mouse(&mut self, msg: u32, info: &MSLLHOOKSTRUCT) -> bool {
        let t0 = now_us();
        if info.dwExtraInfo == TAG {
            // Our input is back, so all input queued before it has passed.
            self.echo_since = None;
            return false;
        }
        let injected = info.flags & LLMHF_INJECTED != 0;
        if !hookio::accept(self.source, injected, info.dwExtraInfo) {
            return false;
        }
        let pt = Pt {
            x: info.pt.x,
            y: info.pt.y,
        };
        let ours = self.own.iter().any(|r| hookio::inside(r, pt));
        let Some(event) = hookio::event_of(msg, pt, ours) else {
            return false;
        };
        let at = self.event_time(info.time, t0);
        let swallow = self.queue(event, at);
        self.note(Some((pt, ours)), now_us() - t0);
        swallow
    }

    /// Handles a key event `msg`; only Esc matters.
    pub fn on_key(&mut self, msg: u32, info: &KBDLLHOOKSTRUCT) {
        let t0 = now_us();
        let injected = info.flags.contains(LLKHF_INJECTED);
        if hookio::is_esc(msg, info.vkCode)
            && hookio::accept(self.source, injected, info.dwExtraInfo)
        {
            let at = self.event_time(info.time, t0);
            self.queue(Event::Esc, at);
            self.note(None, now_us() - t0);
        }
    }

    /// Sets the thread timer for the engine's next deadline, if any.
    #[allow(unsafe_code, reason = "Thread timers of our own thread.")]
    fn reschedule(&mut self, now: i64) {
        // SAFETY: thread timers of our own thread; a zero id is never killed.
        unsafe {
            if self.timer != 0 {
                let _ = KillTimer(None, self.timer);
                self.timer = 0;
            }
            if let Some(due) = self.engine.deadline() {
                self.timer = SetTimer(None, 0, timer_ms(due, now), None);
            }
        }
    }

    /// Applies commands and a due tick; returns the input to inject and whether to stop.
    pub fn turn(&mut self, rx: &Receiver<Call>, replies: &mut Vec<Sender<()>>) -> (Vec<Act>, bool) {
        let mut stop = false;
        loop {
            let acts = match rx.try_recv() {
                Ok((Cmd::Mode(m), reply)) => {
                    replies.push(reply);
                    self.engine.set_mode(m)
                }
                Ok((Cmd::Latch(vk), reply)) => {
                    replies.push(reply);
                    self.engine.latch(vk)
                }
                Ok((Cmd::Stop, reply)) => {
                    replies.push(reply);
                    stop = true;
                    self.engine.reset()
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    stop = true;
                    self.engine.reset()
                }
            };
            self.acts.extend(acts);
            if stop {
                break;
            }
        }
        let now = now_us();
        let tick = self.engine.on(Event::Tick, now);
        self.acts.extend(tick.acts);
        self.reschedule(now);
        (std::mem::take(&mut self.acts), stop)
    }
}

#[cfg(test)]
mod tests;
