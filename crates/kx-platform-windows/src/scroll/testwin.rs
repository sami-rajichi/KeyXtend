//! A hidden test window with scroll bars on a thread of its own: it answers line steps, logs the wheel turns it gets,
//! and can stop reading messages, as a hung app does; or one whose bars split, as a wide table in a long page. It is
//! never shown and never takes the focus.
#![allow(
    unsafe_code,
    reason = "The tests make, scroll and destroy their own hidden windows."
)]

use std::sync::mpsc;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;
use std::time::Duration;

use kx_module_api::ports::{Point, Rect};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Controls::SetScrollInfo;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, GetScrollInfo,
    GetWindowRect, MSG, PostMessageW, PostThreadMessageW, RegisterClassW, SB_HORZ, SB_LINEDOWN,
    SB_LINEUP, SB_VERT, SCROLLBAR_CONSTANTS, SCROLLINFO, SIF_ALL, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_APP, WM_HSCROLL, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_QUIT, WM_VSCROLL, WNDCLASSW, WS_CHILD,
    WS_HSCROLL, WS_POPUP, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::w;

use super::Seek;

/// How long a test waits for the window's thread; a wait, never a sleep.
pub const WAIT: Duration = Duration::from_secs(5);
/// The scroll range: positions 0 to 99 with a page of 10, so 45 is half way.
const RANGE: (i32, i32, u32) = (0, 99, 10);
/// Where both bars start: half way.
const START: i32 = 45;
/// The message a test posts after others, so it knows they were all handled.
const FENCE: u32 = WM_APP;

/// One message the window got: its window, the message and its two words.
pub type Seen = (isize, u32, usize, isize);

/// Keeps the tests that start a UI Automation client apart: clients starting at once in one process can fail their
/// first lookups (`E_FAIL`), and a client stuck on a window that stopped reading slows the others.
static CLIENTS: Mutex<()> = Mutex::new(());

/// Holds the UI Automation tests apart until dropped.
pub fn one_client() -> MutexGuard<'static, ()> {
    CLIENTS.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Every wheel turn and fence the test windows got, oldest first.
static LOG: Mutex<Vec<Seen>> = Mutex::new(Vec::new());
/// Woken at each logged message.
static LOGGED: Condvar = Condvar::new();

/// The window's procedure: line steps move its bars, and wheel turns and fences are logged.
unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_VSCROLL => step(hwnd, SB_VERT, wparam),
        WM_HSCROLL => step(hwnd, SB_HORZ, wparam),
        WM_MOUSEWHEEL | WM_MOUSEHWHEEL | FENCE => {
            let raw = hwnd.0.addr().cast_signed();
            LOG.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push((raw, msg, wparam.0, lparam.0));
            LOGGED.notify_all();
        }
        _ => {}
    }
    // SAFETY: default handling of a message for the test's own window.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Moves bar `bar` one line up or down, as the line step in `wparam` asks.
fn step(hwnd: HWND, bar: SCROLLBAR_CONSTANTS, wparam: WPARAM) {
    let [lo, hi, ..] = wparam.0.to_le_bytes();
    let code = i32::from(u16::from_le_bytes([lo, hi]));
    let by = match code {
        c if c == SB_LINEUP.0 => -1,
        c if c == SB_LINEDOWN.0 => 1,
        _ => 0,
    };
    let mut info = info(0);
    // SAFETY: reads and sets the test window's own bar through a local struct.
    unsafe {
        let _ = GetScrollInfo(hwnd, bar, &raw mut info);
        info.nPos += by;
        SetScrollInfo(hwnd, bar, &raw const info, false);
    }
}

/// The bars' info at position `pos`.
fn info(pos: i32) -> SCROLLINFO {
    let (min, max, page) = RANGE;
    SCROLLINFO {
        cbSize: u32::try_from(size_of::<SCROLLINFO>()).unwrap(),
        fMask: SIF_ALL,
        nMin: min,
        nMax: max,
        nPage: page,
        nPos: pos,
        nTrackPos: 0,
    }
}

/// Which bars a test window has: both, none, or up and down only, with a child at a box of its client that scrolls
/// only sideways.
#[derive(Copy, Clone)]
enum Bars {
    Both,
    None,
    Split(Rect),
}

/// Makes the hidden window at `r` with `bars`, each half way, and gives it and the window reads look at: itself, or its
/// child.
fn make(r: Rect, bars: Bars) -> (HWND, HWND) {
    let style = match bars {
        Bars::Both => WS_POPUP | WS_VSCROLL | WS_HSCROLL,
        Bars::None => WS_POPUP,
        Bars::Split(_) => WS_POPUP | WS_VSCROLL,
    };
    let hwnd = window(r, style, None);
    let Bars::Split(inner) = bars else {
        return (hwnd, hwnd);
    };
    (
        hwnd,
        window(inner, WS_CHILD | WS_VISIBLE | WS_HSCROLL, Some(hwnd)),
    )
}

/// One window of the test class at `r` (in its parent's client when it has one), its bars from `style` half way.
fn window(r: Rect, style: WINDOW_STYLE, parent: Option<HWND>) -> HWND {
    // SAFETY: registers the test class, made again harmlessly, and a window of it that is never shown.
    unsafe {
        let module = GetModuleHandleW(None).unwrap();
        let class = w!("KeyXtend.ScrollTest");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(proc),
            hInstance: module.into(),
            lpszClassName: class,
            ..WNDCLASSW::default()
        };
        RegisterClassW(&raw const wc);
        let ex = WINDOW_EX_STYLE::default();
        let (x, y, cx, cy) = (r.x, r.y, r.w, r.h);
        let title = w!("scroll test");
        let module = Some(module.into());
        let hwnd = CreateWindowExW(
            ex, class, title, style, x, y, cx, cy, parent, None, module, None,
        );
        let hwnd = hwnd.unwrap();
        let half = info(START);
        for (bar, has) in [(SB_VERT, WS_VSCROLL), (SB_HORZ, WS_HSCROLL)] {
            if style.contains(has) {
                SetScrollInfo(hwnd, bar, &raw const half, false);
            }
        }
        hwnd
    }
}

/// The hidden window and its thread; dropping it destroys both.
pub struct TestWin {
    /// The window reads look at: the top-level window, or its child.
    raw: isize,
    /// The top-level window.
    top: isize,
    thread: u32,
    join: Option<JoinHandle<()>>,
    /// Lets a stuck window's thread go on.
    release: Option<mpsc::Sender<()>>,
}

impl TestWin {
    /// The window at `r` with scroll bars, on a thread that reads its messages, or, when `stuck`, stops reading them
    /// once made.
    pub fn start(r: Rect, stuck: bool) -> Self {
        Self::make(r, stuck, Bars::Both)
    }

    /// The window at `r` with no scroll bars, on a thread that reads its messages.
    pub fn plain(r: Rect) -> Self {
        Self::make(r, false, Bars::None)
    }

    /// The window at `r` that scrolls only up and down, holding at `inner` of its client a child, its `raw`, that
    /// scrolls only sideways.
    pub fn split(r: Rect, inner: Rect) -> Self {
        Self::make(r, false, Bars::Split(inner))
    }

    fn make(r: Rect, stuck: bool, bars: Bars) -> Self {
        let (made, ready) = mpsc::channel();
        let (release, wait) = mpsc::channel::<()>();
        let join = std::thread::spawn(move || {
            let (hwnd, seen) = make(r, bars);
            // SAFETY: a plain query of this thread's id.
            let thread = unsafe { GetCurrentThreadId() };
            let raw = |h: HWND| h.0.addr().cast_signed();
            made.send((raw(seen), raw(hwnd), thread)).unwrap();
            if stuck {
                let _ = wait.recv();
            }
            pump();
            // SAFETY: destroys the window this thread made.
            let _ = unsafe { DestroyWindow(hwnd) };
        });
        let (raw, top, thread) = ready.recv_timeout(WAIT).unwrap();
        Self {
            raw,
            top,
            thread,
            join: Some(join),
            release: Some(release),
        }
    }

    /// The window.
    pub fn hwnd(&self) -> HWND {
        crate::window::from_raw(self.raw)
    }

    /// Lets a stuck window's thread read its messages again.
    pub fn release(&mut self) {
        drop(self.release.take());
    }

    /// The window's raw handle.
    pub const fn raw(&self) -> isize {
        self.raw
    }

    /// The window's raw id, as a scroll target names it.
    pub fn id(&self) -> u64 {
        u64::try_from(self.raw.cast_unsigned()).unwrap()
    }

    /// The top-level window's raw id, which is the window's own but for a split one.
    pub fn top_id(&self) -> u64 {
        u64::try_from(self.top.cast_unsigned()).unwrap()
    }

    /// A seek at the middle of the window, inside it.
    pub fn seek(&self) -> Seek {
        let r = self.bounds();
        let mid = Point {
            x: r.x + r.w / 2,
            y: r.y + r.h / 2,
        };
        Seek::Window(self.id(), mid)
    }

    /// The window's box on the screen.
    pub fn bounds(&self) -> Rect {
        let mut r = RECT::default();
        // SAFETY: a box query of the test's own window into a local.
        unsafe { GetWindowRect(self.hwnd(), &raw mut r) }.unwrap();
        crate::screen::rect_of([r.left, r.top, r.right, r.bottom])
    }

    /// The wheel turns the window handled, once every message posted before this call was handled.
    pub fn wheels(&self) -> Vec<Seen> {
        // SAFETY: a message with no data to the test's own window.
        unsafe { PostMessageW(Some(self.hwnd()), FENCE, WPARAM(0), LPARAM(0)) }.unwrap();
        let fenced = |log: &mut Vec<Seen>| !log.iter().any(|s| s.0 == self.raw && s.1 == FENCE);
        let log = LOG.lock().unwrap_or_else(PoisonError::into_inner);
        let (mut log, waited) = LOGGED.wait_timeout_while(log, WAIT, fenced).unwrap();
        assert!(!waited.timed_out(), "the window handled its messages");
        let mine = log
            .iter()
            .filter(|s| s.0 == self.raw && s.1 != FENCE)
            .copied()
            .collect();
        log.retain(|s| s.0 != self.raw);
        mine
    }
}

impl Drop for TestWin {
    fn drop(&mut self) {
        drop(self.release.take());
        // SAFETY: ends the test window's own thread's loop.
        let _ = unsafe { PostThreadMessageW(self.thread, WM_QUIT, WPARAM(0), LPARAM(0)) };
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

/// Reads this thread's messages until `WM_QUIT`.
fn pump() {
    let mut msg = MSG::default();
    // SAFETY: the loop of the thread that owns the window; `msg` is a local it fills.
    while unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.0 > 0 {
        // SAFETY: dispatches a message this thread received.
        unsafe { DispatchMessageW(&raw const msg) };
    }
}
