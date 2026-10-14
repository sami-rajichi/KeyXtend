//! The host's OS calls on Windows: the clock and tick count, the window and screen under a press, posts and timers,
//! `SendInput`, and both hooks, whose handles any thread can take out.

use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};

use kx_module_api::ports::{Button, InjectError, Point, PointerAct};
use kx_module_api::{Clock, Mono};
use windows::Win32::Foundation::{LPARAM, POINT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS, OpenInputDesktop,
};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::{
    GetCurrentThread, GetCurrentThreadId, SetThreadPriority, THREAD_PRIORITY,
    THREAD_PRIORITY_ABOVE_NORMAL,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetLastInputInfo, INPUT, INPUT_0, INPUT_MOUSE, LASTINPUTINFO, MOUSE_EVENT_FLAGS,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN,
    MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, HHOOK, HOOKPROC, KillTimer, MSG, PM_NOREMOVE, PeekMessageW,
    PostThreadMessageW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN, SetTimer, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WINDOWS_HOOK_ID, WindowFromPoint,
};

use super::procs::{key_proc, mouse_proc};
use super::{WAKE, lend::lent};
use crate::QpcClock;
use crate::hook::batch::{Desk, Input, batch};
use crate::hook::stats::Stats;
use crate::hook::{Os, inbox::Inbox};
use crate::inject;
use crate::screen::dpi_of;
use crate::window::is_ours;

/// The hook thread's priority: one step above normal.
const PRIORITY: THREAD_PRIORITY = THREAD_PRIORITY_ABOVE_NORMAL;

/// Each button's flags going down and up.
const BUTTONS: [(Button, MOUSE_EVENT_FLAGS, MOUSE_EVENT_FLAGS); 3] = [
    (Button::Left, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
    (Button::Right, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
    (Button::Middle, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
];

/// What a run's owner and its thread share: the numbers, the hooks' handles and the stop.
#[derive(Default)]
pub(super) struct Ctl {
    /// The run's numbers.
    pub stats: Arc<Stats>,
    /// The mouse and keyboard hooks' handles; 0 once removed, so each is removed once, from whichever thread.
    pub hooks: [AtomicUsize; 2],
    /// The hook thread's id once its queue exists; 0 before.
    pub tid: AtomicU32,
    /// Set by the run's drop or a start that gave up; the loop ends at its next message.
    pub stop: AtomicBool,
}

/// The OS as the host on the hook thread sees it.
pub(super) struct WinOs {
    tid: u32,
    clock: QpcClock,
    ctl: Arc<Ctl>,
    /// The deadline timer's id; 0 when none runs.
    timer: Cell<usize>,
}

impl WinOs {
    /// The OS for the hook thread `tid`.
    pub(super) fn new(tid: u32, clock: QpcClock, ctl: Arc<Ctl>) -> Self {
        Self {
            tid,
            clock,
            ctl,
            timer: Cell::new(0),
        }
    }
}

impl Os for WinOs {
    fn now(&self) -> Mono {
        self.clock.now()
    }

    #[allow(unsafe_code, reason = "A plain query.")]
    fn tick(&self) -> u32 {
        // SAFETY: a plain query.
        unsafe { GetTickCount() }
    }

    #[allow(unsafe_code, reason = "A query into a local struct.")]
    fn last_input(&self) -> Option<u32> {
        let mut info = LASTINPUTINFO {
            cbSize: u32::try_from(size_of::<LASTINPUTINFO>()).ok()?,
            dwTime: 0,
        };
        // SAFETY: `info` is a local sized for the call, which fills it.
        let ok = unsafe { GetLastInputInfo(&raw mut info) }.as_bool();
        ok.then_some(info.dwTime)
    }

    /// Windows refuses to open the secure desktop and the lock screen to a user's program.
    #[allow(
        unsafe_code,
        reason = "Opens the input desktop to test it, then closes it."
    )]
    fn away(&self) -> bool {
        // SAFETY: a plain query that returns a handle we own, or nothing.
        let desk = unsafe {
            OpenInputDesktop(DESKTOP_CONTROL_FLAGS::default(), false, DESKTOP_READOBJECTS)
        };
        let Ok(desk) = desk else {
            return true;
        };
        // SAFETY: the handle opened above, closed once.
        let _ = unsafe { CloseDesktop(desk) };
        false
    }

    #[allow(unsafe_code, reason = "A plain window query.")]
    fn ours(&self, at: Point) -> bool {
        // SAFETY: a plain query by value; it sends no message to another thread's window.
        let hwnd = unsafe { WindowFromPoint(POINT { x: at.x, y: at.y }) };
        is_ours(hwnd)
    }

    fn dpi(&self, at: Point) -> u32 {
        let (right, bottom) = (at.x.saturating_add(1), at.y.saturating_add(1));
        dpi_of(&RECT {
            left: at.x,
            top: at.y,
            right,
            bottom,
        })
    }

    fn wake(&self) -> bool {
        post(self.tid, WAKE)
    }

    fn send(&self, acts: &[PointerAct], first: u16, inbox: &mut Inbox) -> usize {
        let all = batch(acts, desktop(), first);
        let inputs: Vec<INPUT> = all.into_iter().map(input).collect();
        match lent(inbox, || inject::send(&inputs)) {
            Ok(()) => inputs.len(),
            Err(InjectError::Refused { sent, .. }) => sent,
            Err(InjectError::Unmappable) => 0,
        }
    }

    fn arm(&self, ms: Option<u32>) -> bool {
        kill_timer(self.timer.take());
        self.timer.set(ms.map_or(0, set_timer));
        ms.is_none() || self.timer.get() != 0
    }

    fn hook(&self) -> Result<(), i32> {
        install(&self.ctl)
    }

    fn unhook(&self) {
        unhook_all(&self.ctl);
    }
}

/// Starts a timer of this thread that fires every `ms`; its id, or 0 when Windows refused.
#[allow(unsafe_code, reason = "A timer of this thread, with no callback.")]
pub(super) fn set_timer(ms: u32) -> usize {
    // SAFETY: a thread timer with no window and no callback; it posts `WM_TIMER` to this thread.
    unsafe { SetTimer(None, 0, ms, None) }
}

/// Stops this thread's timer `id`; 0 is none.
#[allow(unsafe_code, reason = "A timer of this thread.")]
pub(super) fn kill_timer(id: usize) {
    if id != 0 {
        // SAFETY: a timer this thread set; a gone one is refused.
        let _ = unsafe { KillTimer(None, id) };
    }
}

/// Installs the mouse hook, then the keyboard hook; when the second is refused, the first comes out again.
#[allow(unsafe_code, reason = "Our own module handle.")]
fn install(ctl: &Ctl) -> Result<(), i32> {
    // SAFETY: a plain query of our own program module.
    let module = unsafe { GetModuleHandleW(None) }.map_err(|e| e.code().0)?;
    let set = |slot: usize, id: WINDOWS_HOOK_ID, proc: HOOKPROC| {
        // SAFETY: our own procedures, which live as long as the program, in our own module.
        let hook = unsafe { SetWindowsHookExW(id, proc, Some(module.into()), 0) };
        let hook = hook.map_err(|e| e.code().0)?;
        if let Some(handle) = ctl.hooks.get(slot) {
            handle.store(hook.0.addr(), Ordering::SeqCst);
        }
        Ok(())
    };
    set(0, WH_MOUSE_LL, Some(mouse_proc))?;
    set(1, WH_KEYBOARD_LL, Some(key_proc)).inspect_err(|_| unhook_all(ctl))
}

/// Removes the hooks `ctl` holds, each once, from whichever thread takes it first.
#[allow(unsafe_code, reason = "Removes hooks this process installed.")]
pub(super) fn unhook_all(ctl: &Ctl) {
    for handle in &ctl.hooks {
        let raw = handle.swap(0, Ordering::SeqCst);
        if raw != 0 {
            // SAFETY: a hook this process installed, taken out of its slot so it is removed only once.
            let _ = unsafe { UnhookWindowsHookEx(HHOOK(std::ptr::without_provenance_mut(raw))) };
        }
    }
}

/// The virtual desktop in physical pixels.
#[allow(unsafe_code, reason = "Plain metric queries.")]
fn desktop() -> Desk {
    // SAFETY: plain metric queries.
    unsafe {
        Desk {
            left: GetSystemMetrics(SM_XVIRTUALSCREEN),
            top: GetSystemMetrics(SM_YVIRTUALSCREEN),
            width: GetSystemMetrics(SM_CXVIRTUALSCREEN),
            height: GetSystemMetrics(SM_CYVIRTUALSCREEN),
        }
    }
}

/// `one` as a Windows input.
fn input(one: Input) -> INPUT {
    match one {
        Input::Mouse { dx, dy, press, tag } => INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: 0,
                    dwFlags: flags(press),
                    time: 0,
                    dwExtraInfo: tag,
                },
            },
        },
        Input::Key { event, tag } => inject::input(event, tag),
    }
}

/// An absolute move over the virtual desktop, with `press`'s button going down, or up when its flag is set.
fn flags(press: Option<(Button, bool)>) -> MOUSE_EVENT_FLAGS {
    let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    let edge = press.and_then(|(button, up)| {
        let (_, down_flag, up_flag) = BUTTONS.iter().find(|(b, ..)| *b == button)?;
        Some(if up { *up_flag } else { *down_flag })
    });
    at | edge.unwrap_or_default()
}

/// Creates this thread's message queue before anyone posts to it, and gives the thread's id.
#[allow(
    unsafe_code,
    reason = "Creates this thread's queue, then a plain query."
)]
pub(super) fn own_queue() -> u32 {
    let mut msg = MSG::default();
    // SAFETY: a peek into a local that only creates this thread's queue, then a plain query.
    unsafe {
        let _ = PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE);
        GetCurrentThreadId()
    }
}

/// Raises this thread one step, so Windows is less likely to drop a hook under load; a refusal changes nothing.
#[allow(unsafe_code, reason = "Sets the calling thread's own priority.")]
pub(super) fn raise_priority() {
    // SAFETY: the pseudo handle of the calling thread, which needs no closing.
    let _ = unsafe { SetThreadPriority(GetCurrentThread(), PRIORITY) };
}

/// This thread's id.
#[allow(unsafe_code, reason = "A plain thread query.")]
pub(super) fn this_thread() -> u32 {
    // SAFETY: a plain query of the calling thread.
    unsafe { GetCurrentThreadId() }
}

/// Posts `msg` to thread `tid`; false when Windows refused it.
#[allow(unsafe_code, reason = "Posts a message that carries no pointer.")]
pub(super) fn post(tid: u32, msg: u32) -> bool {
    // SAFETY: the message carries no pointer, so it is sound whichever thread gets it, even one that took the id of a
    // thread that ended.
    unsafe { PostThreadMessageW(tid, msg, WPARAM(0), LPARAM(0)) }.is_ok()
}

#[cfg(test)]
mod tests;
