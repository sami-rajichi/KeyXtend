//! The hooks' callbacks: each reads what Windows passes, asks the host, and passes the event on unless the engine
//! drops a mouse event. No panic crosses into Windows, with no host every event passes, and a busy host is inside our
//! own send, so our echo goes to the inbox it lent and the user's event is counted there.

use std::panic::{AssertUnwindSafe, catch_unwind};

use kx_module_api::ports::Point;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLMHF_INJECTED,
    MSLLHOOKSTRUCT,
};

use super::lend::hear;
use super::with_host;
use crate::hook::Mark;
use crate::hook::accept::{key_act, kind, sent_scan};

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
    let answer = ask(mark, kind(msg).is_some(), || {
        with_host(|h| h.mouse(msg, at, mark))
    });
    answer.is_some_and(|dropped| dropped)
}

/// Hands a key event to the host; it always passes on.
fn key(msg: u32, info: &KBDLLHOOKSTRUCT) {
    let mark = Mark {
        injected: info.flags.contains(LLKHF_INJECTED),
        extra: info.dwExtraInfo,
        time: info.time,
    };
    let scan = sent_scan(info.scanCode, info.flags.contains(LLKHF_EXTENDED));
    let _ = ask(mark, key_act(msg, scan).is_some(), || {
        with_host(|h| h.key(msg, info.vkCode, scan, mark))
    });
}

/// Runs the host's `call` so that no panic crosses into Windows, which would end the program. When the host does not
/// answer, the event with `mark` goes to the inbox our send lent, if any; `read` says the host would read it.
fn ask<T>(mark: Mark, read: bool, call: impl FnOnce() -> Option<T>) -> Option<T> {
    let run = || {
        let answer = call();
        if answer.is_none() {
            hear(mark, read);
        }
        answer
    };
    catch_unwind(AssertUnwindSafe(run)).ok().flatten()
}

/// The message id a hook passes in `wparam`; 0, which no event uses, if it does not fit.
fn msg_id(wparam: WPARAM) -> u32 {
    u32::try_from(wparam.0).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::{WM_KEYDOWN, WM_MOUSEMOVE, WM_MOUSEWHEEL};

    use super::super::lend::with_inbox;
    use super::*;
    use crate::hook::Source;
    use crate::hook::inbox::Inbox;
    use crate::keys::replay_tag;

    /// Room for every event this test hears, so only a missing tag keeps one out.
    const ACTS: usize = 2;

    #[test]
    fn with_no_host_to_answer_our_echoes_pass_and_go_to_the_lent_inbox_in_order() {
        let ours = MSLLHOOKSTRUCT {
            pt: POINT { x: 1, y: 2 },
            flags: LLMHF_INJECTED,
            dwExtraInfo: replay_tag(3),
            ..MSLLHOOKSTRUCT::default()
        };
        let our_key = KBDLLHOOKSTRUCT {
            flags: LLKHF_INJECTED,
            dwExtraInfo: replay_tag(4),
            ..KBDLLHOOKSTRUCT::default()
        };
        let mut inbox = Inbox::new(ACTS, Source::Physical);
        with_inbox(&mut inbox, || {
            assert!(!mouse(WM_MOUSEMOVE, &ours), "our echo passes");
            key(WM_KEYDOWN, &our_key);
            let hand = MSLLHOOKSTRUCT::default();
            assert!(!mouse(WM_MOUSEMOVE, &hand), "the hand's event passes");
            assert!(!mouse(WM_MOUSEWHEEL, &hand), "a wheel passes");
            key(WM_KEYDOWN, &KBDLLHOOKSTRUCT::default());
        });
        let ids: Vec<_> = inbox.drain().collect();
        assert_eq!(ids, [3, 4], "untagged events keep nothing");
        assert_eq!(
            inbox.take_passed(),
            2,
            "the hand's move and key, not its wheel"
        );
    }
}
