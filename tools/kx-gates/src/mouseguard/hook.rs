//! The mouse guard's hook: a low-level mouse hook on a thread of its own that applies the guard's filter. Windows
//! removes it when that thread or kx-gates ends, so it can never outlive a run.
#![cfg(windows)]

use std::cell::RefCell;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, OnceLock};
use std::thread::JoinHandle;

use kx_module_api::Clock;
use kx_platform_windows::QpcClock;
use windows::Win32::Foundation::{LPARAM, LRESULT, WAIT_FAILED, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HC_ACTION, HHOOK, MSG, MSLLHOOKSTRUCT, MsgWaitForMultipleObjects, PM_NOREMOVE,
    PM_REMOVE, PeekMessageW, PostThreadMessageW, QS_ALLINPUT, SetWindowsHookExW,
    UnhookWindowsHookEx, WH_MOUSE_LL, WM_APP,
};

use super::run::{Hook, Live};
use super::{Counts, End, Filter, Verdict, kind_of, ours, wait_ms};
use crate::out::say;

/// Tells Windows to drop the event.
const SWALLOW: LRESULT = LRESULT(1);
/// The hook `code` that carries an event.
const ACTION: i32 = HC_ACTION.cast_signed();
/// Asks the hook thread to let go.
const STOP: u32 = WM_APP;
/// The hook thread's name.
const THREAD: &str = "kx-mouse-guard";

/// What the hook reads, on the hook thread only.
struct State {
    filter: Filter,
    clock: QpcClock,
}

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

/// Runs `f` on the hook thread's state; none when there is none or it is busy, and the event then passes.
fn with_state<T>(f: impl FnOnce(&mut State) -> T) -> Option<T> {
    STATE.with(|s| s.try_borrow_mut().ok()?.as_mut().map(f))
}

#[allow(unsafe_code, reason = "`lparam` is a valid MSLLHOOKSTRUCT here.")]
unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == ACTION {
        // SAFETY: for HC_ACTION, `lparam` points to an MSLLHOOKSTRUCT that lives for this call.
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        let mine = ours(info.flags, info.dwExtraInfo);
        let kind = kind_of(u32::try_from(wparam.0).unwrap_or_default(), info.mouseData);
        let verdict = with_state(|s| s.filter.event(mine, kind, s.clock.now()));
        if verdict == Some(Verdict::Drop) {
            return SWALLOW;
        }
    }
    // SAFETY: passes the event on unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Our hook, removed when dropped, so even an unwinding thread lets go.
struct Installed(HHOOK);

impl Installed {
    #[allow(unsafe_code, reason = "Our module handle and hook procedure.")]
    fn new() -> Result<Self, String> {
        // SAFETY: our own module handle, and a hook procedure that lives for the whole program.
        unsafe {
            let module = GetModuleHandleW(None).map_err(|e| format!("GetModuleHandleW: {e}"))?;
            SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), Some(module.into()), 0)
                .map(Self)
                .map_err(|e| format!("mouse guard hook: {e}"))
        }
    }
}

impl Drop for Installed {
    #[allow(unsafe_code, reason = "Removes our own hook.")]
    fn drop(&mut self) {
        // SAFETY: our own hook, removed once.
        let _ = unsafe { UnhookWindowsHookEx(self.0) };
    }
}

/// Creates this thread's message queue, so `STOP` can reach it; the thread's id.
#[allow(unsafe_code, reason = "A peek and a query on this thread.")]
fn queue() -> u32 {
    let mut msg = MSG::default();
    // SAFETY: a peek that creates this thread's queue before anyone posts to it, then a plain query.
    unsafe {
        let _ = PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE);
        GetCurrentThreadId()
    }
}

/// Handles messages and hook calls until `STOP` comes or the filter lets go; why it let go, if it did, even when
/// `STOP` came in the same batch. A wait that fails lets go too, as the guard could no longer check its time.
#[allow(unsafe_code, reason = "This thread's own message queue.")]
fn pump(clock: QpcClock) -> Option<End> {
    let mut msg = MSG::default();
    loop {
        // SAFETY: drains this thread's queue; hook calls run inside the peek.
        while unsafe { PeekMessageW(&raw mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
            if msg.message == STOP {
                return with_state(|s| s.filter.ended()).flatten();
            }
        }
        let now = clock.now();
        let due = match with_state(|s| (s.filter.tick(now), s.filter.due_in(now))) {
            Some((None, due)) => due,
            Some((end, _)) => return end,
            None => return Some(End::Lost),
        };
        // SAFETY: waits on no handles for a message, a hook call or the next check.
        let woke = unsafe { MsgWaitForMultipleObjects(None, false, wait_ms(due), QS_ALLINPUT) };
        if woke == WAIT_FAILED {
            return Some(End::Lost);
        }
    }
}

/// The hook thread: installs the hook, sends its id, pumps, then removes the hook and says why when it let go itself;
/// what its filter counted.
fn run(state: State, ended: &OnceLock<End>, ready: &Sender<Result<u32, String>>) -> Counts {
    let (tid, clock) = (queue(), state.clock);
    STATE.with(|s| *s.borrow_mut() = Some(state));
    let hook = match Installed::new() {
        Ok(hook) => hook,
        Err(e) => {
            let _ = ready.send(Err(e));
            return Counts::default();
        }
    };
    let _ = ready.send(Ok(tid));
    let own = pump(clock);
    drop(hook);
    if let Some(end) = own {
        let _ = ended.set(end);
        say!("{}", end.line());
    }
    with_state(|s| s.filter.counts()).unwrap_or_default()
}

/// Starts the real hook, on a thread of its own.
pub struct WinHook {
    clock: QpcClock,
}

impl WinHook {
    /// A hook that reads time from `clock`, the guard's own.
    pub fn new(clock: QpcClock) -> Self {
        Self { clock }
    }
}

impl Hook for WinHook {
    type Live = Running;

    fn start(&self, filter: Filter) -> Result<Running, String> {
        let ended = Arc::new(OnceLock::new());
        let (tx, rx) = mpsc::channel();
        let (state, mine) = (
            State {
                filter,
                clock: self.clock,
            },
            Arc::clone(&ended),
        );
        let thread = std::thread::Builder::new()
            .name(THREAD.to_string())
            .spawn(move || run(state, &mine, &tx))
            .map_err(|e| format!("mouse guard thread: {e}"))?;
        let tid = rx
            .recv()
            .unwrap_or_else(|_| Err("the mouse guard thread ended".to_string()));
        match tid {
            Ok(tid) => Ok(Running {
                tid,
                thread: Some(thread),
                ended,
            }),
            Err(e) => {
                let _ = thread.join();
                Err(e)
            }
        }
    }
}

/// A hook thread that runs; dropping it stops the thread, so no hook outlives its owner.
pub struct Running {
    tid: u32,
    thread: Option<JoinHandle<Counts>>,
    ended: Arc<OnceLock<End>>,
}

impl Running {
    /// Asks the hook thread to let go and waits for it, once; what it counted, none the second time.
    #[allow(unsafe_code, reason = "Posts to our own hook thread.")]
    fn halt(&mut self) -> Counts {
        let Some(thread) = self.thread.take() else {
            return Counts::default();
        };
        // SAFETY: our own thread, whose id stays ours while `thread` holds its handle; a post to a thread that already
        // ended only fails.
        let _ = unsafe { PostThreadMessageW(self.tid, STOP, WPARAM(0), LPARAM(0)) };
        thread.join().unwrap_or_default()
    }
}

impl Live for Running {
    fn ended(&self) -> Option<End> {
        self.ended.get().copied()
    }

    fn stop(mut self) -> (Option<End>, Counts) {
        let counts = self.halt();
        (self.ended.get().copied(), counts)
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        self.halt();
    }
}
