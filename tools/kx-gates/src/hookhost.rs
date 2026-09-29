//! The assist's hook thread: low-level hooks, the message loop, and injection after each hook returns.
#![cfg(windows)]

use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use spike_core::clock::now_us;
use spike_core::hold::Act;
use spike_core::inject::{self, TAG};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT,
    PM_NOREMOVE, PeekMessageW, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_TIMER,
};

use crate::assist::Setup;
use crate::hookio;
use crate::hookstate::{Call, Host, Live, Report, WAKE};
use crate::win;

/// Tells Windows to drop the event.
const SWALLOW: LRESULT = LRESULT(1);
/// The hook `code` that carries an event.
const ACTION: i32 = HC_ACTION.cast_signed();

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

/// Runs `f` on the host, unless there is none or it is busy (then the event just passes).
fn with_host<T>(f: impl FnOnce(&mut Host) -> T) -> Option<T> {
    HOST.with(|h| h.try_borrow_mut().ok()?.as_mut().map(f))
}

/// The message id a hook passes in `wparam`; 0, which no event uses, if it does not fit.
fn msg_id(wparam: WPARAM) -> u32 {
    u32::try_from(wparam.0).unwrap_or_default()
}

#[allow(unsafe_code, reason = "`lparam` is a valid MSLLHOOKSTRUCT here.")]
unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == ACTION {
        // SAFETY: for HC_ACTION, `lparam` points to an MSLLHOOKSTRUCT that lives for this call.
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        if with_host(|h| h.on_mouse(msg_id(wparam), info)).unwrap_or(false) {
            return SWALLOW;
        }
    }
    // SAFETY: passes the event on unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

#[allow(unsafe_code, reason = "`lparam` is a valid KBDLLHOOKSTRUCT here.")]
unsafe extern "system" fn key_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == ACTION {
        // SAFETY: for HC_ACTION, `lparam` points to a KBDLLHOOKSTRUCT that lives for this call.
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        with_host(|h| h.on_key(msg_id(wparam), info));
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
        with_host(|h| h.sent(now_us()));
    }
    if let Err(e) = inject::send(&hookio::inputs(acts, win::desktop(), TAG)) {
        with_host(|h| h.failed(e));
    }
}

/// One loop turn: applies commands, injects, then answers the callers; true to stop.
fn turn(rx: &Receiver<Call>) -> bool {
    let mut replies = Vec::new();
    let (acts, stop) = with_host(|h| h.turn(rx, &mut replies)).unwrap_or((Vec::new(), true));
    inject_acts(&acts);
    for reply in replies {
        let _ = reply.send(());
    }
    stop
}

/// The hook thread: installs the hooks, sends its id, loops until `Stop` or quit, and lets go of everything.
#[allow(unsafe_code, reason = "Queries, and creates this thread's queue.")]
pub fn run(
    setup: Setup,
    rx: &Receiver<Call>,
    live: Arc<Mutex<Live>>,
    ready: &Sender<Result<u32, String>>,
) -> Report {
    let mut msg = MSG::default();
    // SAFETY: a plain query, then creating this thread's queue before anyone posts to it.
    let tid = unsafe {
        let _ = PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE);
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
    while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
        if matches!(msg.message, WAKE | WM_TIMER) && turn(rx) {
            break;
        }
    }
    for hook in hooks {
        // SAFETY: our own hooks, removed once.
        let _ = unsafe { UnhookWindowsHookEx(hook) };
    }
    // Whatever ended the loop, nothing stays pressed.
    inject_acts(&with_host(Host::reset).unwrap_or_default());
    HOST.with(|h| h.borrow_mut().take())
        .map(Host::into_report)
        .unwrap_or_default()
}
