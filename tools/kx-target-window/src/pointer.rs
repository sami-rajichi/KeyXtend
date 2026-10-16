//! The edit box's pointer lines: mouse presses and wheel turns, each with the screen point its message carried.

use kx_target_window::log::{self, Press};
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    WM_CONTEXTMENU, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEWHEEL,
    WM_RBUTTONDBLCLK, WM_RBUTTONDOWN, WM_RBUTTONUP,
};

use crate::procs::{log_line, now_us};

/// Bits in one word of a mouse message's `lparam`: x sits in the low word, y in the next; a wheel's `wparam` holds
/// its turn in the high word.
const WORD_BITS: u32 = 16;

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

/// Whether a wheel message turns the sideways wheel, and its signed turn; none for any other message.
fn wheel_of(msg: u32, wparam: WPARAM) -> Option<(bool, i32)> {
    let sideways = match msg {
        WM_MOUSEWHEEL => false,
        WM_MOUSEHWHEEL => true,
        _ => return None,
    };
    Some((sideways, word((wparam.0 >> WORD_BITS).cast_signed())))
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

/// Logs message `msg` when it is a press or a wheel turn; true for a context-menu request, which the box swallows.
pub fn note(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
    if let Some((sideways, delta)) = wheel_of(msg, wparam) {
        // A wheel message's point is on screen already, in the window's own pixels.
        let pt = point_of(lparam.0);
        log_line(&log::wheel(now_us(), sideways, delta, pt.x, pt.y));
    }
    let Some((press, on_screen)) = press_of(msg) else {
        return false;
    };
    log_press(hwnd, press, on_screen, lparam);
    press == Press::Menu
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::WindowsAndMessaging::WM_CHAR;

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
    fn a_wheel_message_carries_its_signed_turn_in_the_high_word() {
        let turn = |delta: i16| WPARAM(usize::from(delta.cast_unsigned()) << WORD_BITS);
        assert_eq!(wheel_of(WM_MOUSEWHEEL, turn(-120)), Some((false, -120)));
        assert_eq!(wheel_of(WM_MOUSEHWHEEL, turn(40)), Some((true, 40)));
        assert_eq!(wheel_of(WM_CHAR, turn(120)), None);
    }
}
