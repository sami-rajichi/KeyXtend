//! Tests of the Windows side's batch and queries; none installs a hook, sends input or moves the pointer.

use kx_module_api::ports::Scan;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE,
};

use super::*;
use crate::keys::REPLAY;

/// How many desktop queries show that none leaks a handle; tests running beside it open far fewer.
const QUERIES: u32 = 1_000;

/// The mouse fields of `one`, which must be a mouse input.
#[allow(unsafe_code, reason = "Reads the member the type field names.")]
fn mouse_of(one: &INPUT) -> MOUSEINPUT {
    assert_eq!(one.r#type, INPUT_MOUSE);
    // SAFETY: a mouse input, so `mi` is the live member.
    unsafe { one.Anonymous.mi }
}

/// The key fields of `one`, which must be a keyboard input.
#[allow(unsafe_code, reason = "Reads the member the type field names.")]
fn key_of(one: &INPUT) -> KEYBDINPUT {
    assert_eq!(one.r#type, INPUT_KEYBOARD);
    // SAFETY: a keyboard input, so `ki` is the live member.
    unsafe { one.Anonymous.ki }
}

#[test]
fn each_button_goes_down_and_up_by_its_own_flag_at_an_absolute_point() {
    let at = MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK;
    assert_eq!(flags(None), at);
    let want = [
        (Button::Left, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        (Button::Right, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        (Button::Middle, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    ];
    for (button, down, up) in want {
        assert_eq!(flags(Some((button, false))), at | down, "{button:?}");
        assert_eq!(flags(Some((button, true))), at | up, "{button:?}");
    }
}

#[test]
fn a_batch_goes_out_as_absolute_mouse_and_scan_code_keys_tagged_replay() {
    let p = Point { x: 5, y: 6 };
    let right_alt = Scan::new(0xE038);
    let acts = [
        PointerAct::Down(Button::Right, p),
        PointerAct::KeyUp(right_alt),
    ];
    let desk = Desk {
        left: 0,
        top: 0,
        width: 11,
        height: 11,
    };
    let got: Vec<INPUT> = batch(&acts, desk).into_iter().map(input).collect();
    let mi = mouse_of(&got[0]);
    assert_eq!((mi.dx, mi.dy, mi.dwExtraInfo), (32768, 39321, REPLAY));
    assert_eq!(mi.dwFlags, flags(Some((Button::Right, false))));
    let ki = key_of(&got[1]);
    assert_eq!((ki.wVk.0, ki.wScan, ki.dwExtraInfo), (0, 0x38, REPLAY));
    let up = KEYEVENTF_SCANCODE | KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP;
    assert_eq!(ki.dwFlags, up);
}

#[test]
fn the_queries_answer_without_a_hook() {
    let os = WinOs::new(0, QpcClock::new().unwrap(), Arc::default());
    let desk = desktop();
    assert!(desk.width > 0 && desk.height > 0);
    let corner = Point {
        x: desk.left,
        y: desk.top,
    };
    assert!(os.dpi(corner) > 0);
    assert!(!os.ours(corner), "no window of ours is shown");
    assert!(os.last_input().is_some());
    assert!(os.now() > Mono::default());
}

#[test]
fn removing_hooks_that_are_not_there_does_nothing() {
    let ctl = Ctl::default();
    unhook_all(&ctl);
    assert!(ctl.hooks.iter().all(|h| h.load(Ordering::SeqCst) == 0));
}

#[test]
#[allow(unsafe_code, reason = "Reads the calling thread's own priority.")]
fn the_hook_thread_runs_one_step_above_normal() {
    use windows::Win32::System::Threading::{GetCurrentThread, GetThreadPriority};
    let got = std::thread::spawn(|| {
        raise_priority();
        // SAFETY: the pseudo handle of the calling thread, which needs no closing.
        unsafe { GetThreadPriority(GetCurrentThread()) }
    })
    .join()
    .unwrap();
    assert_eq!(got, PRIORITY.0);
}

#[test]
fn the_deadline_timer_is_set_and_stopped_on_this_thread() {
    let os = WinOs::new(0, QpcClock::new().unwrap(), Arc::default());
    assert!(os.arm(Some(60_000)));
    assert_ne!(os.timer.get(), 0);
    assert!(os.arm(None));
    assert_eq!(os.timer.get(), 0);
}

#[test]
#[allow(unsafe_code, reason = "Reads this process's handle count.")]
fn the_desktop_query_closes_the_handle_it_opens() {
    use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessHandleCount};
    let count = || {
        let mut n = 0;
        // SAFETY: the pseudo handle of this process, which needs no closing, and a local the call fills.
        unsafe { GetProcessHandleCount(GetCurrentProcess(), &raw mut n) }.unwrap();
        n
    };
    let os = WinOs::new(0, QpcClock::new().unwrap(), Arc::default());
    let before = count();
    for _ in 0..QUERIES {
        let _ = os.away();
    }
    assert!(count() < before.saturating_add(QUERIES / 2));
}
