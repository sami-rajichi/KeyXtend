//! The window procedures and the log they write to.

use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use kx_module_api::{Clock, Mono};
use kx_platform_windows::QpcClock;
use kx_target_window::hang;
use kx_target_window::layout::LayoutError;
use kx_target_window::log::{self, Press};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::DefSubclassProc;
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GetClientRect, MoveWindow, PostQuitMessage, WM_CHAR, WM_CLOSE, WM_CONTEXTMENU,
    WM_DESTROY, WM_KEYDOWN, WM_KILLFOCUS, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETFOCUS, WM_SIZE, WM_SYSKEYDOWN,
};

use crate::switch;

/// The log file, shared with the edit-box subclass.
static LOG: OnceLock<Mutex<File>> = OnceLock::new();
/// The clock the log's times come from, the same QPC time base kx-gates uses.
static CLOCK: OnceLock<QpcClock> = OnceLock::new();
/// The edit box, as a raw handle value.
static EDIT: AtomicIsize = AtomicIsize::new(0);
/// Set in password mode: typed characters are never logged.
static PASSWORD: AtomicBool = AtomicBool::new(false);
/// The longest sleep a hang message may ask for, in ms; 0 until `set_hang_max`.
static HANG_MAX_MS: AtomicU64 = AtomicU64::new(0);
/// Bits in one word of a mouse message's `lparam`: x sits in the low word, y in the next.
const WORD_BITS: u32 = 16;
/// The prefix byte of an extended key's scan code, as `kx_module_api::ports::Scan` writes it (`0xE0xx`).
const EXTENDED_PREFIX: u8 = 0xE0;
/// Bit 24 of a key message's `lparam`, the lowest bit of its fourth byte: an extended key.
const EXTENDED_FLAG: u8 = 1;

/// Keeps `file` as the log and `clock` as its time source, then writes the start line.
pub fn start_log(file: File, clock: QpcClock) {
    let _ = LOG.set(Mutex::new(file));
    let _ = CLOCK.set(clock);
    log_line(&log::mark(log::START, now_us()));
}

/// The time for a log line in microseconds; 0 before `start_log`.
fn now_us() -> i64 {
    CLOCK.get().map_or(0, |clock| log_us(clock.now()))
}

/// `at` as the log writes it: signed microseconds, stopping at the largest.
fn log_us(at: Mono) -> i64 {
    i64::try_from(at.as_us()).unwrap_or(i64::MAX)
}

/// Logs `# layout-kept` when `e` left the session typing in another layout, so a gate sees it.
pub fn note_layout(e: &LayoutError) {
    if e.kept() {
        log_line(&log::mark(log::LAYOUT_KEPT, now_us()));
    }
}

/// Turns password mode on or off.
pub fn set_password(on: bool) {
    PASSWORD.store(on, Ordering::Relaxed);
}

/// Sets the longest sleep a gate's hang message may ask for, in ms.
pub fn set_hang_max(ms: u64) {
    HANG_MAX_MS.store(ms, Ordering::Relaxed);
}

/// Sleeps the time a hang message asks in `wparam`, capped by `set_hang_max`, as a hung app does.
fn hang_for(wparam: WPARAM) {
    let asked = u64::try_from(wparam.0).unwrap_or(u64::MAX);
    let ms = hang::capped(asked, HANG_MAX_MS.load(Ordering::Relaxed));
    std::thread::sleep(Duration::from_millis(ms));
}

/// Remembers `edit` as the logging edit box.
pub fn set_edit(edit: HWND) {
    EDIT.store(edit.0 as isize, Ordering::Relaxed);
}

/// The logging edit box; null until `set_edit` runs.
fn edit() -> HWND {
    HWND(EDIT.load(Ordering::Relaxed) as *mut _)
}

/// Appends `line` to the log and flushes it, so a killed window loses nothing.
fn log_line(line: &str) {
    if let Some(file) = LOG.get()
        && let Ok(mut f) = file.lock()
    {
        let _ = writeln!(f, "{line}");
        let _ = f.flush();
    }
}

/// The low word of `wparam`: a `WM_CHAR`'s UTF-16 unit, or a key message's virtual key.
fn low_word(wparam: WPARAM) -> u16 {
    let [lo, hi, ..] = wparam.0.to_le_bytes();
    u16::from_le_bytes([lo, hi])
}

/// The scan code a key message carries in bits 16-23 of `lparam`, with the extended prefix when bit 24 is set.
fn scan_of(lparam: isize) -> u16 {
    let [_, _, scan, flags, ..] = lparam.to_le_bytes();
    let prefix = if flags & EXTENDED_FLAG == 0 {
        0
    } else {
        EXTENDED_PREFIX
    };
    u16::from_be_bytes([prefix, scan])
}

/// The signed low word of `v`.
fn word(v: isize) -> i32 {
    let [lo, hi, ..] = v.to_le_bytes();
    i32::from(i16::from_le_bytes([lo, hi]))
}

/// The signed x and y packed in a mouse message's `lparam`.
fn point_of(lparam: isize) -> POINT {
    POINT {
        x: word(lparam),
        y: word(lparam >> WORD_BITS),
    }
}

/// The press a mouse message means, and whether its point is already on screen.
fn press_of(msg: u32) -> Option<(Press, bool)> {
    match msg {
        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => Some((Press::LeftDown, false)),
        WM_LBUTTONUP => Some((Press::LeftUp, false)),
        WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => Some((Press::RightDown, false)),
        WM_RBUTTONUP => Some((Press::RightUp, false)),
        WM_CONTEXTMENU => Some((Press::Menu, true)),
        _ => None,
    }
}

/// Logs `press` at its screen point; client points of `hwnd` are converted first.
#[allow(unsafe_code, reason = "ClientToScreen on our own edit box.")]
fn log_press(hwnd: HWND, press: Press, on_screen: bool, lparam: LPARAM) {
    let mut pt = point_of(lparam.0);
    if !on_screen {
        // SAFETY: plain conversion on our own edit box; a failure leaves the client point.
        let _ = unsafe { ClientToScreen(hwnd, &raw mut pt) };
    }
    log_line(&log::mouse(press, now_us(), pt.x, pt.y));
}

/// The edit box's subclass procedure: logs characters, keys going down, focus losses and presses; a password box logs
/// no character and no key.
#[allow(
    unsafe_code,
    reason = "Win32 subclass callback that forwards to DefSubclassProc."
)]
pub extern "system" fn edit_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_CHAR if !PASSWORD.load(Ordering::Relaxed) => {
            log_line(&log::data(now_us(), low_word(wparam)));
        }
        WM_KEYDOWN | WM_SYSKEYDOWN if !PASSWORD.load(Ordering::Relaxed) => {
            log_line(&log::key(now_us(), low_word(wparam), scan_of(lparam.0)));
        }
        WM_KILLFOCUS => log_line(&log::mark(log::FOCUS_LOST, now_us())),
        _ => {}
    }
    if let Some((press, on_screen)) = press_of(msg) {
        log_press(hwnd, press, on_screen, lparam);
        if press == Press::Menu {
            // No menu: a test right-click must leave nothing open.
            return LRESULT(0);
        }
    }
    // SAFETY: forwards the same message to the edit box.
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// Gives `hwnd` the keyboard focus.
#[allow(unsafe_code, reason = "SetFocus on our own window.")]
pub fn focus(hwnd: HWND) {
    // SAFETY: plain call on our own window; a failure leaves the focus where it was.
    let _ = unsafe { SetFocus(Some(hwnd)) };
}

/// Makes the edit box fill the client area of `main`.
#[allow(unsafe_code, reason = "Win32 calls on our own windows.")]
pub fn fit_edit(main: HWND) {
    let mut rc = RECT::default();
    // SAFETY: plain calls on our own windows; a null edit handle only makes MoveWindow fail.
    unsafe {
        if GetClientRect(main, &raw mut rc).is_ok() {
            let _ = MoveWindow(edit(), 0, 0, rc.right - rc.left, rc.bottom - rc.top, true);
        }
    }
}

/// The main window's procedure: keeps the edit box sized and focused, sleeps when a gate asks, and on close puts the
/// layout back and ends the loop.
#[allow(
    unsafe_code,
    reason = "Win32 window procedure with standard calls on our own windows."
)]
pub extern "system" fn main_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_SIZE => {
            fit_edit(hwnd);
            LRESULT(0)
        }
        WM_SETFOCUS => {
            focus(edit());
            LRESULT(0)
        }
        WM_CLOSE => {
            // The layout goes back while the window still has the keyboard; then Windows destroys it.
            if let Err(e) = switch::undo() {
                note_layout(&e);
            }
            // SAFETY: the default handling of a message our own window received.
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        m if m == hang::MESSAGE => {
            hang_for(wparam);
            LRESULT(0)
        }
        WM_DESTROY => {
            // SAFETY: posts the quit message to this thread's own loop.
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        // SAFETY: the default handling of a message our own window received.
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_of_reads_signed_words() {
        let pack = |x: i16, y: i16| {
            let word = |v: i16| usize::from(v.cast_unsigned());
            ((word(y) << WORD_BITS) | word(x)).cast_signed()
        };
        let p = point_of(pack(-5, 300));
        assert_eq!((p.x, p.y), (-5, 300));
        let p = point_of(pack(1200, -2));
        assert_eq!((p.x, p.y), (1200, -2));
    }

    #[test]
    fn press_of_maps_buttons_and_the_menu() {
        assert_eq!(press_of(WM_LBUTTONDBLCLK), Some((Press::LeftDown, false)));
        assert_eq!(press_of(WM_RBUTTONUP), Some((Press::RightUp, false)));
        assert_eq!(press_of(WM_CONTEXTMENU), Some((Press::Menu, true)));
        assert_eq!(press_of(WM_CHAR), None);
    }

    #[test]
    fn log_times_are_signed_microseconds_that_stop_at_the_largest() {
        assert_eq!(log_us(Mono::from_us(42)), 42);
        assert_eq!(log_us(Mono::from_us(i64::MAX.unsigned_abs())), i64::MAX);
        assert_eq!(log_us(Mono::from_us(u64::MAX)), i64::MAX);
    }

    #[test]
    fn a_char_message_carries_one_utf16_unit_in_its_low_word() {
        assert_eq!(low_word(WPARAM(0x0644)), 0x0644);
        assert_eq!(low_word(WPARAM(0xFFFF_0041)), 0x0041);
    }

    #[test]
    fn a_key_message_carries_its_scan_code_as_the_product_writes_it() {
        use kx_module_api::ports::Scan;
        // Enter: scan 0x1C in bits 16-23, a repeat count of 1, and the previous-state bit set.
        assert_eq!(scan_of(0x401C_0001), 0x001C);
        // Up arrow: scan 0x48 with bit 24, the extended key.
        let up = Scan::new(scan_of(0x0148_0001));
        assert!(up.is_extended() && up.byte() == 0x48, "{up:?}");
        assert!(!Scan::new(scan_of(0x002A_0001)).is_extended());
    }
}
