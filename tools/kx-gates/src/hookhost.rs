//! The assist's hook thread: low-level hooks, the hold timer, and injection after each hook returns.
//!
//! Input the engine lets through never overtakes our own queued input: it waits behind it until that comes back.
#![cfg(windows)]

use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use spike_core::clock::now_us;
use spike_core::hold::{Act, Engine, Event, Mode, Pt};
use spike_core::inject::{self, TAG};
use windows::Win32::Foundation::{LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, KillTimer, LLKHF_INJECTED,
    LLMHF_INJECTED, MSG, MSLLHOOKSTRUCT, PM_NOREMOVE, PeekMessageW, PostThreadMessageW, SetTimer,
    SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_APP, WM_TIMER,
};

use crate::assist::Setup;
use crate::hookio::{self, Source};
use crate::{stats, win};

/// Wakes the hook thread: commands or injections are waiting.
pub const WAKE: u32 = WM_APP + 1;
/// Tells Windows to drop the event.
const SWALLOW: LRESULT = LRESULT(1);
/// One extra ms, so the hold timer never fires before its deadline.
const ROUND_UP_MS: i64 = 1;
/// Most hook timings kept; later calls still run but are not timed.
const HOOK_SAMPLES: usize = 1 << 16;
/// Microseconds per millisecond, as an integer.
const US_PER_MS: i64 = stats::US_PER_MS as i64;

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
struct Host {
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

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

/// Runs `f` on the host, unless there is none or it is busy (then the event just passes).
fn with_host<T>(f: impl FnOnce(&mut Host) -> T) -> Option<T> {
    HOST.with(|h| h.try_borrow_mut().ok()?.as_mut().map(f))
}

/// Milliseconds from `now` until `due` (both µs), rounded up so the timer is never early.
fn timer_ms(due: i64, now: i64) -> u32 {
    let ms = due.saturating_sub(now).max(0) / US_PER_MS;
    u32::try_from(ms.saturating_add(ROUND_UP_MS)).unwrap_or(u32::MAX)
}

impl Host {
    fn new(setup: Setup, tid: u32, live: Arc<Mutex<Live>>) -> Self {
        let us = |ms: u64| {
            i64::try_from(ms)
                .ok()
                .and_then(|ms| ms.checked_mul(US_PER_MS))
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
        now - i64::from(waited) * US_PER_MS
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

    fn on_mouse(&mut self, msg: u32, info: &MSLLHOOKSTRUCT) -> bool {
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

    fn on_key(&mut self, msg: u32, info: &KBDLLHOOKSTRUCT) {
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
    fn turn(&mut self, rx: &Receiver<Call>, acks: &mut Vec<Sender<()>>) -> (Vec<Act>, bool) {
        let mut stop = false;
        loop {
            let acts = match rx.try_recv() {
                Ok((Cmd::Mode(m), ack)) => {
                    acks.push(ack);
                    self.engine.set_mode(m)
                }
                Ok((Cmd::Latch(vk), ack)) => {
                    acks.push(ack);
                    self.engine.latch(vk)
                }
                Ok((Cmd::Stop, ack)) => {
                    acks.push(ack);
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
        let step = self.engine.on(Event::Tick, now);
        self.acts.extend(step.acts);
        self.reschedule(now);
        (std::mem::take(&mut self.acts), stop)
    }
}

#[allow(unsafe_code, reason = "`lparam` is a valid MSLLHOOKSTRUCT here.")]
unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for HC_ACTION, `lparam` points to an MSLLHOOKSTRUCT that lives for this call.
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        if with_host(|h| h.on_mouse(wparam.0 as u32, info)).unwrap_or(false) {
            return SWALLOW;
        }
    }
    // SAFETY: passes the event on unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

#[allow(unsafe_code, reason = "`lparam` is a valid KBDLLHOOKSTRUCT here.")]
unsafe extern "system" fn key_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: for HC_ACTION, `lparam` points to a KBDLLHOOKSTRUCT that lives for this call.
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        with_host(|h| h.on_key(wparam.0 as u32, info));
    }
    // SAFETY: passes the event on unchanged; Esc is never swallowed.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Installs both hooks on this thread.
#[allow(unsafe_code, reason = "Our module handle and hook procedures.")]
fn install() -> Result<[HHOOK; 2], String> {
    // SAFETY: our own module handle and hook procedures that live for the whole program.
    unsafe {
        let module = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW: {e}"))?;
        let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), Some(module.into()), 0)
            .map_err(|e| format!("mouse hook: {e}"))?;
        match SetWindowsHookExW(WH_KEYBOARD_LL, Some(key_proc), Some(module.into()), 0) {
            Ok(keys) => Ok([mouse, keys]),
            Err(e) => {
                let _ = UnhookWindowsHookEx(mouse);
                Err(format!("keyboard hook: {e}"))
            }
        }
    }
}

/// Injects `acts` as one batch; mouse input starts the wait for its echo, set first so an early echo counts.
fn inject_acts(acts: &[Act]) {
    if acts.is_empty() {
        return;
    }
    if acts.iter().any(hookio::is_mouse) {
        with_host(|h| h.echo_since = Some(now_us()));
    }
    if let Err(e) = inject::send(&hookio::inputs(acts, win::desktop(), TAG)) {
        with_host(|h| {
            h.echo_since = None;
            h.report.errors.push(e);
        });
    }
}

/// One loop turn: applies commands, injects, then answers the callers; true to stop.
fn turn(rx: &Receiver<Call>) -> bool {
    let mut acks = Vec::new();
    let (acts, stop) = with_host(|h| h.turn(rx, &mut acks)).unwrap_or((Vec::new(), true));
    inject_acts(&acts);
    for ack in acks {
        let _ = ack.send(());
    }
    stop
}

/// The hook thread: installs the hooks, sends its id, loops until `Stop` or quit, and lets go of everything.
#[allow(unsafe_code, reason = "Queries, and creates this thread's queue.")]
pub fn run(
    setup: Setup,
    rx: Receiver<Call>,
    live: Arc<Mutex<Live>>,
    ready: Sender<Result<u32, String>>,
) -> Report {
    let mut msg = MSG::default();
    // SAFETY: a plain query, then creating this thread's queue before anyone posts to it.
    let tid = unsafe {
        let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
        GetCurrentThreadId()
    };
    HOST.with(|h| *h.borrow_mut() = Some(Host::new(setup, tid, live)));
    let hooks = match install() {
        Ok(hooks) => hooks,
        Err(e) => {
            let _ = ready.send(Err(e));
            return Report::default();
        }
    };
    let _ = ready.send(Ok(tid));
    // SAFETY: a plain message loop on this thread; it ends on WM_QUIT (0) or an error (-1).
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
        if matches!(msg.message, WAKE | WM_TIMER) && turn(&rx) {
            break;
        }
    }
    for hook in hooks {
        // SAFETY: our own hooks, removed once.
        let _ = unsafe { UnhookWindowsHookEx(hook) };
    }
    // Whatever ended the loop, nothing stays pressed.
    inject_acts(&with_host(|h| h.engine.reset()).unwrap_or_default());
    HOST.with(|h| h.borrow_mut().take())
        .map(|h| h.report)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_ms_rounds_up_and_never_overflows() {
        assert_eq!(timer_ms(1_500_000, 0), 1501);
        assert_eq!(timer_ms(1_500_999, 0), 1501);
        assert_eq!(timer_ms(0, 10), 1);
        assert_eq!(timer_ms(i64::MAX, i64::MIN), u32::MAX);
    }
}
