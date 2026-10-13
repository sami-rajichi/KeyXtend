//! The hooks' callbacks: each reads what Windows passes, asks the host, and passes the event on unless the engine
//! drops a mouse event. No panic crosses into Windows, and with no host every event passes.

use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_module_api::ports::Point;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED, MSLLHOOKSTRUCT,
};

use super::with_host;
use crate::hook::Mark;

/// Tells Windows to drop the event.
const SWALLOW: LRESULT = LRESULT(1);
/// The hook `code` that carries an event.
const ACTION: i32 = HC_ACTION.cast_signed();

#[allow(unsafe_code, reason = "Reads the struct Windows passes the hook.")]
pub(super) unsafe extern "system" fn mouse_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code == ACTION {
        let ptr = std::ptr::with_exposed_provenance::<MSLLHOOKSTRUCT>(lparam.0.cast_unsigned());
        // SAFETY: for HC_ACTION, `lparam` points to an MSLLHOOKSTRUCT that lives for this call.
        let info = unsafe { ptr.as_ref() };
        if info.is_some_and(|info| mouse(msg_id(wparam), info)) {
            return SWALLOW;
        }
    }
    // SAFETY: passes the event on unchanged.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

#[allow(unsafe_code, reason = "Reads the struct Windows passes the hook.")]
pub(super) unsafe extern "system" fn key_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code == ACTION {
        let ptr = std::ptr::with_exposed_provenance::<KBDLLHOOKSTRUCT>(lparam.0.cast_unsigned());
        // SAFETY: for HC_ACTION, `lparam` points to a KBDLLHOOKSTRUCT that lives for this call.
        if let Some(info) = unsafe { ptr.as_ref() } {
            key(msg_id(wparam), info);
        }
    }
    // SAFETY: passes the event on unchanged; no key is ever dropped.
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// The host's answer for a mouse event: true to drop it. With no host, a busy one or a panic, it passes.
fn mouse(msg: u32, info: &MSLLHOOKSTRUCT) -> bool {
    let at = Point {
        x: info.pt.x,
        y: info.pt.y,
    };
    let mark = Mark {
        injected: info.flags & LLMHF_INJECTED != 0,
        extra: info.dwExtraInfo,
        time: info.time,
    };
    shield(|| with_host(|h| h.mouse(msg, at, mark))).is_some_and(|dropped| dropped)
}

/// Hands a key event to the host; it always passes on.
fn key(msg: u32, info: &KBDLLHOOKSTRUCT) {
    let mark = Mark {
        injected: info.flags.contains(LLKHF_INJECTED),
        extra: info.dwExtraInfo,
        time: info.time,
    };
    let _ = shield(|| with_host(|h| h.key(msg, info.vkCode, mark)));
}

/// Runs `call` so that no panic crosses into Windows, which would end the program.
fn shield<T>(call: impl FnOnce() -> Option<T>) -> Option<T> {
    catch_unwind(AssertUnwindSafe(call)).ok().flatten()
}

/// The message id a hook passes in `wparam`; 0, which no event uses, if it does not fit.
fn msg_id(wparam: WPARAM) -> u32 {
    u32::try_from(wparam.0).unwrap_or_default()
}
