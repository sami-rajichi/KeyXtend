//! The Windows desk for the scroll run (ADR-0029): the pointer, the window at a point and whose it is, hung and closed
//! windows, the wheel settings, posted and sent wheel turns, and UI Automation on its own thread. Points are physical
//! pixels, as the process is per-monitor DPI aware.

use std::time::Duration;

use kx_module_api::ports::{
    Dir, Point, Rect, Screen, ScrollConfig, ScrollError, ScrollRun, ScrollTarget,
};
use kx_module_api::{Clock, Mono};
use kx_platform::ui::Screens;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::HiDpi::PhysicalToLogicalPointForPerMonitorDPI;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_WHEEL, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOT, GetAncestor, GetSystemMetrics, IsHungAppWindow, IsWindow,
    MOUSEWHEEL_ROUTING_MOUSE_POS, PostMessageW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN,
    SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SPI_GETMOUSEWHEELROUTING, SPI_GETWHEELSCROLLCHARS,
    SPI_GETWHEELSCROLLLINES, SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    SystemParametersInfoW, WHEEL_DELTA, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WindowFromPoint,
};

mod pinger;

use super::entry::Kept;
use super::uia::{Seek, Uia};
use super::{Desk, Read, Run, Spot, Wheel, Whose, point_lparam, wheel_wparam};
use crate::keys::TAG;
use crate::window::{entry, from_raw, is_ours};
use crate::{QpcClock, WinScreens, screen};
use pinger::Pinger;

/// The Windows scroll target: each run reads and turns windows where the user pointed.
#[derive(Clone, Copy, Debug)]
pub struct WinScroll {
    clock: QpcClock,
}

impl WinScroll {
    /// A target whose runs read the time from `clock`.
    #[must_use]
    pub const fn new(clock: QpcClock) -> Self {
        Self { clock }
    }
}

impl ScrollTarget for WinScroll {
    fn open(&self, config: ScrollConfig) -> Result<Box<dyn ScrollRun>, ScrollError> {
        let desk = WinDesk::new(self.clock, config);
        Ok(Box::new(Run::new(desk, config)))
    }
}

/// Windows' side of one run: the crate's clock, and the run's UI Automation and ping threads.
struct WinDesk {
    clock: QpcClock,
    uia: Uia,
    pinger: Pinger,
}

impl WinDesk {
    /// The desk for a run as `config` says, with its threads started.
    fn new(clock: QpcClock, config: ScrollConfig) -> Self {
        Self {
            clock,
            uia: Uia::start(config.read_ms),
            pinger: Pinger::start(config.read_ms, config.check_ms),
        }
    }
}

impl Desk for WinDesk {
    fn cursor(&self) -> Option<Point> {
        screen::cursor().ok()
    }

    fn spot(&self, p: Point) -> Option<Spot> {
        spot(p)
    }

    fn kept(&self) -> Option<Kept> {
        entry::kept()
    }

    fn screens(&self) -> Vec<Screen> {
        WinScreens.work_areas()
    }

    fn whole(&self) -> Rect {
        whole()
    }

    fn alive(&self, window: u64) -> bool {
        alive(window)
    }

    fn hung(&self, window: u64) -> bool {
        hung(window) || self.pinger.busy(window)
    }

    fn ping(&self, window: u64) {
        self.pinger.ask(window);
    }

    fn wheel(&self) -> Wheel {
        wheel()
    }

    fn post(&self, window: u64, at: Point, dir: Dir, delta: i32) -> Result<(), ScrollError> {
        post(window, at, dir, delta)
    }

    fn send(&self, dir: Dir, delta: i32) -> Result<(), ScrollError> {
        crate::inject::send(&[wheel_input(dir, delta)]).map_err(|_| ScrollError::Refused)
    }

    fn read(&self, at: Point, cap: Duration) -> Option<Read> {
        self.uia.read(Seek::At(at), cap)
    }

    fn idle(&self) -> bool {
        self.uia.idle()
    }

    fn steps(&self, at: Point, dir: Dir, count: u16) -> bool {
        self.uia.steps(Seek::At(at), dir, count)
    }

    fn now(&self) -> Mono {
        self.clock.now()
    }
}

/// Window `hwnd` as a raw id.
fn id_of(hwnd: HWND) -> u64 {
    u64::try_from(hwnd.0.addr()).unwrap_or_default()
}

/// The window raw id `window` names.
fn handle(window: u64) -> HWND {
    from_raw(usize::try_from(window).unwrap_or_default().cast_signed())
}

/// The window at `p` and whose it is: the keyboard the guard watches, another of ours, or another app's.
#[allow(unsafe_code, reason = "Plain window queries.")]
fn spot(p: Point) -> Option<Spot> {
    // SAFETY: plain queries; no window there gives null.
    let (hwnd, root) = unsafe {
        let hwnd = WindowFromPoint(POINT { x: p.x, y: p.y });
        (hwnd, GetAncestor(hwnd, GA_ROOT))
    };
    if hwnd.is_invalid() {
        return None;
    }
    let whose = Whose::of(is_ours(root), entry::watched(root));
    let window = id_of(hwnd);
    Some(Spot { window, whose })
}

/// The box around every screen.
#[allow(unsafe_code, reason = "Plain system metric reads.")]
fn whole() -> Rect {
    // SAFETY: plain reads; an unknown metric gives 0.
    let metric = |index| unsafe { GetSystemMetrics(index) };
    Rect {
        x: metric(SM_XVIRTUALSCREEN),
        y: metric(SM_YVIRTUALSCREEN),
        w: metric(SM_CXVIRTUALSCREEN),
        h: metric(SM_CYVIRTUALSCREEN),
    }
}

/// True while window `window` exists.
#[allow(unsafe_code, reason = "A plain window query.")]
fn alive(window: u64) -> bool {
    // SAFETY: a plain query; a gone window reads as no window.
    unsafe { IsWindow(Some(handle(window))) }.as_bool()
}

/// True while Windows reports `window`'s app as hung; it never waits on that app.
#[allow(unsafe_code, reason = "A plain window query.")]
fn hung(window: u64) -> bool {
    // SAFETY: a plain query that reads Windows' own record, sending the window nothing.
    unsafe { IsHungAppWindow(handle(window)) }.as_bool()
}

/// The user's wheel settings now, with Windows' notch.
fn wheel() -> Wheel {
    Wheel {
        notch: WHEEL_DELTA,
        lines: setting(SPI_GETWHEELSCROLLLINES),
        chars: setting(SPI_GETWHEELSCROLLCHARS),
        at_pointer: setting(SPI_GETMOUSEWHEELROUTING) == MOUSEWHEEL_ROUTING_MOUSE_POS,
    }
}

/// One wheel setting; 0 when Windows will not say, which goes by UI Automation's steps and the focus's routing.
#[allow(unsafe_code, reason = "A system setting read into a local.")]
fn setting(action: SYSTEM_PARAMETERS_INFO_ACTION) -> u32 {
    let mut value = 0u32;
    let data = std::ptr::from_mut(&mut value).cast();
    let keep = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default();
    // SAFETY: the call writes one u32 into a local that outlives it.
    let read = unsafe { SystemParametersInfoW(action, 0, Some(data), keep) };
    if read.is_ok() { value } else { 0 }
}

/// Posts a wheel turn of `delta` on `dir`'s wheel to `window`, at screen point `at` in the window's own pixels.
#[allow(unsafe_code, reason = "Posts a message with no pointers.")]
fn post(window: u64, at: Point, dir: Dir, delta: i32) -> Result<(), ScrollError> {
    let msg = if dir.sideways() {
        WM_MOUSEHWHEEL
    } else {
        WM_MOUSEWHEEL
    };
    let at = logical(window, at);
    let (w, l) = (WPARAM(wheel_wparam(delta)), LPARAM(point_lparam(at)));
    // SAFETY: the message carries only numbers; a gone window refuses it.
    let posted = unsafe { PostMessageW(Some(handle(window)), msg, w, l) };
    posted.map_err(|_| {
        if alive(window) {
            ScrollError::Refused
        } else {
            ScrollError::Gone
        }
    })
}

/// Physical point `at` in the pixels of `window`, which may not be DPI aware; as it is when Windows cannot say.
#[allow(unsafe_code, reason = "A point conversion into a local.")]
fn logical(window: u64, at: Point) -> Point {
    let mut p = POINT { x: at.x, y: at.y };
    // SAFETY: a local point and a plain query; a gone window refuses it.
    let done = unsafe { PhysicalToLogicalPointForPerMonitorDPI(Some(handle(window)), &raw mut p) };
    if done.as_bool() {
        Point { x: p.x, y: p.y }
    } else {
        at
    }
}

/// One wheel turn of `delta` on `dir`'s wheel at the pointer, tagged as ours so our hook lets it pass.
fn wheel_input(dir: Dir, delta: i32) -> INPUT {
    let flags = if dir.sideways() {
        MOUSEEVENTF_HWHEEL
    } else {
        MOUSEEVENTF_WHEEL
    };
    let mi = MOUSEINPUT {
        mouseData: delta.cast_unsigned(),
        dwFlags: flags,
        dwExtraInfo: TAG,
        ..MOUSEINPUT::default()
    };
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 { mi },
    }
}

#[cfg(test)]
mod tests;
