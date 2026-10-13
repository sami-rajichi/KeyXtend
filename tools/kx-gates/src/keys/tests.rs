//! Tests of the gates' keys and raw input: extended keys, combos, and what goes out once the mouse guard stopped a gate.

use kx_platform_windows::keys::TAG;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    KEYEVENTF_EXTENDEDKEY, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_VIRTUALDESK, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEINPUT,
};

use super::*;
use crate::mouse::{input, normalise};
use crate::mouseguard::fake::{Fake, cfg, held, slot_lock};

#[allow(unsafe_code, reason = "Reads the keyboard member of a key input.")]
fn flags(vk: u16, up: bool) -> (bool, bool, usize) {
    // SAFETY: reading the keyboard member of an input built as a keyboard input.
    let ki = unsafe { tagged(vk, up, TAG).Anonymous.ki };
    let (f, tag) = (ki.dwFlags, ki.dwExtraInfo);
    (
        f.contains(KEYEVENTF_EXTENDEDKEY),
        f.contains(KEYEVENTF_KEYUP),
        tag,
    )
}

/// A mouse input with `flags` at an absolute spot, as the gates send it.
fn at(flags: MOUSE_EVENT_FLAGS) -> INPUT {
    input(5, 6, MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | flags, TAG)
}

/// The mouse member of mouse input `i`.
#[allow(unsafe_code, reason = "Reads the mouse member of a mouse input.")]
fn mouse_of(i: &INPUT) -> MOUSEINPUT {
    assert_eq!(i.r#type, INPUT_MOUSE);
    // SAFETY: the type, just checked, says the mouse member is set.
    unsafe { i.Anonymous.mi }
}

#[test]
fn navigation_keys_are_sent_as_extended_so_numlock_cannot_turn_them_into_digits() {
    for vk in [0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x2D, 0x2E, 0x21, 0x22] {
        assert_eq!(flags(vk, true), (true, true, TAG), "{vk:#04X}");
    }
}

#[test]
fn letters_and_left_modifiers_are_not_extended() {
    for vk in [0x41, 0x10, 0x11, 0x12] {
        assert_eq!(flags(vk, false), (false, false, TAG), "{vk:#04X}");
    }
}

#[test]
fn only_a_batch_that_lets_go_and_presses_nothing_passes_a_stopped_guard_without_its_move() {
    let (moved, down, up) = (
        at(MOUSE_EVENT_FLAGS(0)),
        at(MOUSEEVENTF_LEFTDOWN),
        at(MOUSEEVENTF_LEFTUP),
    );
    assert!(only_lets_go(&[moved, up]), "a release where it moved");
    assert!(only_lets_go(&[
        tagged(0x10, true, TAG),
        at(MOUSEEVENTF_RIGHTUP)
    ]));
    assert!(!only_lets_go(&[moved, down, up]), "a click");
    assert!(!only_lets_go(&[moved]), "a move alone");
    assert!(
        !only_lets_go(&[input(0, 0, MOUSEEVENTF_WHEEL, TAG)]),
        "a wheel turn"
    );
    assert!(!only_lets_go(&[tagged(0x41, false, TAG)]), "a key press");
    assert!(!only_lets_go(&[]), "nothing to let go of");
    let shift_up = tagged(0x10, true, TAG);
    let ups = outgoing(&[moved, up, shift_up], Err("stopped".into()), After::Still);
    let ups = ups.expect("a release goes out");
    assert_eq!(ups.len(), 2, "the move alone is gone");
    let mi = mouse_of(&ups[0]);
    assert_eq!((mi.dx, mi.dy, mi.dwExtraInfo), (0, 0, TAG));
    assert_eq!(mi.dwFlags, MOUSEEVENTF_LEFTUP, "no move, no absolute");
    assert_eq!(
        press_release(&ups[1]),
        (false, true),
        "the key still goes up"
    );
}

#[test]
fn a_stop_between_the_gates_press_and_its_release_still_lets_go_away_from_the_key() {
    let _slot = slot_lock();
    let fake = Fake::new();
    let scope = held(&fake);
    let (moved, down, up) = (
        at(MOUSE_EVENT_FLAGS(0)),
        at(MOUSEEVENTF_LEFTDOWN),
        at(MOUSEEVENTF_LEFTUP),
    );
    let press = outgoing(&[moved, down], mouseguard::allowed(), After::Away);
    assert_eq!(press.map(|p| p.len()), Ok(2), "the press went out");
    fake.wait(cfg().limit_ms);
    let away = outgoing(&[moved, up], mouseguard::allowed(), After::Away);
    let flags: Vec<_> = away
        .expect("the release goes out")
        .iter()
        .map(|i| mouse_of(i).dwFlags)
        .collect();
    let (to, off) = (mouse_of(&moved).dwFlags, mouse_of(&up).dwFlags);
    assert_eq!(flags, [to, off], "it moves away first, so no click lands");
    let again = outgoing(&[moved, down], mouseguard::allowed(), After::Away);
    assert!(again.is_err(), "no new press after the stop");
    assert!(scope.finish(true).is_some_and(|o| o.end.stops()));
}

#[test]
fn only_a_batch_that_presses_or_releases_a_mouse_button_is_watched_as_own_clicks() {
    let moved = at(MOUSE_EVENT_FLAGS(0));
    let click = [moved, at(MOUSEEVENTF_LEFTDOWN), at(MOUSEEVENTF_LEFTUP)];
    assert!(clicks(&click), "a click");
    assert!(clicks(&[at(MOUSEEVENTF_RIGHTUP)]), "a release");
    assert!(clicks(&[at(MOUSEEVENTF_XDOWN)]), "a side button");
    assert!(!clicks(&[moved]), "a move may end where the pointer is");
    let wheel = input(0, 0, MOUSEEVENTF_WHEEL, TAG);
    assert!(!clicks(&[wheel]), "a wheel turn");
    assert!(!clicks(&[tagged(0x41, false, TAG)]), "a key");
    assert!(!clicks(&[]));
}

#[test]
fn a_move_that_lands_far_enough_from_the_pointer_is_watched_and_one_beside_it_is_not() {
    let desk = (-1920, 0, 3840, 1080);
    let (left, _, width, height) = desk;
    let to = |x, y| {
        let (x, y) = (normalise(x, left, width), normalise(y, 0, height));
        let flags = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
        input(x, y, flags, TAG)
    };
    let (at, near, far) = (-100, -100 + SURE_MOVE_PX - 1, -100 + SURE_MOVE_PX);
    let here = POINT { x: at, y: 500 };
    assert!(!moves_far(&[to(at, 500)], here, desk), "where it is");
    assert!(
        !moves_far(&[to(near, 500)], here, desk),
        "may round onto it"
    );
    assert!(moves_far(&[to(far, 500)], here, desk));
    assert!(moves_far(&[to(at, 500 - SURE_MOVE_PX)], here, desk));
    assert!(
        moves_far(&[to(at, 500), to(far, 500)], here, desk),
        "a later step"
    );
    let relative = input(5, 5, MOUSEEVENTF_MOVE, TAG);
    let after = [relative, to(far, 500)];
    assert!(!moves_far(&after, here, desk), "the pointer is unknown");
    assert!(!moves_far(&[tagged(0x41, false, TAG)], here, desk), "a key");
}

#[test]
fn only_a_batch_with_an_absolute_move_needs_the_pointer_read() {
    let keys = [tagged(0x41, false, TAG), tagged(0x41, true, TAG)];
    assert!(!moves_absolute(&keys), "G1's key bursts");
    assert!(!moves_absolute(&[input(5, 5, MOUSEEVENTF_MOVE, TAG)]));
    assert!(moves_absolute(&[at(MOUSE_EVENT_FLAGS(0))]));
}

#[test]
fn a_combo_presses_in_order_and_releases_in_reverse() {
    let got: Vec<_> = presses(&[0x11, 0x43]).iter().map(|e| e.up).collect();
    assert_eq!(got, [false, false, true, true]);
    let keys: Vec<_> = presses(&[0x11, 0x43]).iter().map(|e| e.key).collect();
    assert_eq!(
        keys,
        [vk_key(0x11), vk_key(0x43), vk_key(0x43), vk_key(0x11)]
    );
}
