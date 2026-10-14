//! Tests of the host's pure parts: whose input counts, messages as events, timing and the batch.

use kx_module_api::Mono;
use kx_module_api::ports::{Button, Point, PointerAct, Scan};

use super::ms;
use crate::hook::Source;
use crate::hook::accept::{Kind, Take, act, esc, key_act, kind, take};
use crate::hook::batch::{Desk, Input, batch, normalise};
use crate::hook::timing::{Histogram, event_at, timer_ms};
use crate::keys::{REPLAY, SIM_TAG, TAG};
use crate::strokes::Key;

/// Another program's tag.
const OTHER: usize = 0x1234;
/// `WM_KEYDOWN`, `WM_KEYUP` and `WM_SYSKEYDOWN`.
const KEY_DOWN: u32 = 0x0100;
const KEY_UP: u32 = 0x0101;
const SYS_DOWN: u32 = 0x0104;
/// `VK_ESCAPE` and `VK_RETURN`.
const VK_ESC: u32 = 0x1B;
const VK_ENTER: u32 = 0x0D;
/// A callback well inside the budget, and one near its end, in µs.
const FAST_US: u64 = 25;
const SLOW_US: u64 = 900;

#[test]
fn our_input_passes_as_an_echo_whatever_the_source() {
    for source in [Source::Physical, Source::Simulated] {
        for tag in [TAG, REPLAY] {
            assert_eq!(take(source, true, tag), Take::Echo, "{source:?}");
            assert_eq!(take(source, false, tag), Take::Echo, "{source:?}");
        }
    }
}

#[test]
fn only_the_hand_reaches_the_engine_in_physical_mode() {
    assert_eq!(take(Source::Physical, false, 0), Take::Engine);
    for tag in [0, OTHER, SIM_TAG] {
        assert_eq!(take(Source::Physical, true, tag), Take::Pass, "{tag:#x}");
    }
}

#[test]
fn only_the_gate_tools_input_reaches_the_engine_in_simulated_mode() {
    assert_eq!(take(Source::Simulated, true, SIM_TAG), Take::Engine);
    assert_eq!(take(Source::Simulated, false, 0), Take::Pass, "the hand");
    assert_eq!(take(Source::Simulated, true, OTHER), Take::Pass);
}

#[test]
fn esc_counts_from_the_hand_or_our_keyboard_and_never_from_another_program() {
    let phys = |msg, vk, injected, extra| esc(Source::Physical, msg, vk, injected, extra);
    assert!(phys(KEY_DOWN, VK_ESC, false, 0), "the hand");
    assert!(phys(SYS_DOWN, VK_ESC, false, 0), "with Alt held");
    assert!(phys(KEY_DOWN, VK_ESC, true, TAG), "our keyboard");
    assert!(!phys(KEY_DOWN, VK_ESC, true, OTHER), "another program");
    assert!(
        !phys(KEY_DOWN, VK_ESC, true, REPLAY),
        "the engine's own Esc"
    );
    assert!(!phys(KEY_UP, VK_ESC, false, 0), "a release");
    assert!(!phys(KEY_DOWN, VK_ENTER, false, 0), "another key");
}

#[test]
fn esc_counts_only_from_the_gate_tool_in_simulated_mode() {
    let sim = |injected, extra| esc(Source::Simulated, KEY_DOWN, VK_ESC, injected, extra);
    assert!(sim(true, SIM_TAG));
    assert!(!sim(false, 0) && !sim(true, TAG) && !sim(true, REPLAY));
}

#[test]
fn mouse_messages_become_kinds_and_wheels_have_none() {
    assert_eq!(kind(0x0200), Some(Kind::Move));
    assert_eq!(kind(0x0201), Some(Kind::Down(Button::Left)));
    assert_eq!(kind(0x0205), Some(Kind::Up(Button::Right)));
    assert_eq!(kind(0x0207), Some(Kind::Down(Button::Middle)));
    for wheel_or_side in [0x020A, 0x020B, 0x020E, 0] {
        assert_eq!(kind(wheel_or_side), None, "{wheel_or_side:#x}");
    }
}

#[test]
fn our_events_coming_back_read_as_the_acts_we_sent() {
    let (at, shift) = (Point { x: 5, y: 6 }, Scan::new(0x2A));
    assert_eq!(
        act(Kind::Up(Button::Middle), at),
        PointerAct::Up(Button::Middle, at)
    );
    assert_eq!(act(Kind::Move, at), PointerAct::Move(at));
    assert_eq!(key_act(KEY_DOWN, shift), Some(PointerAct::KeyDown(shift)));
    assert_eq!(key_act(SYS_DOWN, shift), Some(PointerAct::KeyDown(shift)));
    assert_eq!(key_act(KEY_UP, shift), Some(PointerAct::KeyUp(shift)));
    assert_eq!(key_act(0x0102, shift), None, "WM_CHAR");
}

#[test]
fn a_deadline_timer_never_fires_early() {
    let now = ms(10);
    assert_eq!(timer_ms(Mono::from_us(now.as_us() + 1_200), now), 2);
    assert_eq!(timer_ms(ms(1_510), now), 1_501);
    assert_eq!(timer_ms(ms(9), now), 1, "a past deadline");
    assert_eq!(timer_ms(Mono::from_us(u64::MAX), Mono::default()), u32::MAX);
}

#[test]
fn an_event_happened_when_its_tick_says() {
    let (at, waited) = event_at(ms(5_000), 700, 690);
    assert_eq!((at, waited), (ms(4_990), 10));
    let (at, waited) = event_at(ms(5_000), 3, u32::MAX - 6);
    assert_eq!((at, waited), (ms(4_990), 10), "across the tick's wrap");
    assert_eq!(
        event_at(ms(5_000), 690, 700),
        (ms(5_000), 0),
        "a stamp ahead"
    );
    assert_eq!(
        event_at(ms(1), 9_000, 0).0,
        Mono::default(),
        "never before zero"
    );
}

#[test]
fn the_p99_reads_the_fast_range_until_one_in_a_hundred_is_slow() {
    let fill = |fast: u64, slow: u64| {
        let h = Histogram::new();
        (0..fast).for_each(|_| h.record(FAST_US));
        (0..slow).for_each(|_| h.record(SLOW_US));
        h
    };
    let ok = fill(990, 10);
    assert!(
        ok.p99() > 0 && u64::from(ok.p99()) <= FAST_US * 2,
        "{}",
        ok.p99()
    );
    assert_eq!(u64::from(ok.max_us()), SLOW_US);
    let slow = fill(990, 20);
    assert!(u64::from(slow.p99()) >= SLOW_US, "{}", slow.p99());
    assert_eq!(Histogram::new().p99(), 0, "no samples");
}

#[test]
fn a_p99_past_the_last_bucket_reads_the_longest_callback() {
    let h = Histogram::new();
    h.record(FAST_US);
    h.record(50_000);
    assert_eq!((h.p99(), h.max_us()), (50_000, 50_000));
}

#[test]
fn normalise_maps_the_desktop_edges() {
    assert_eq!(normalise(0, 0, 1920), 0);
    assert_eq!(normalise(1919, 0, 1920), 65535);
    assert_eq!(normalise(-1920, -1920, 3840), 0);
    assert_eq!(normalise(5, 0, 11), 32768);
}

#[test]
fn a_batch_is_absolute_mouse_and_scan_code_keys_all_tagged_replay() {
    let desk = Desk {
        left: -1920,
        top: 0,
        width: 3840,
        height: 1080,
    };
    let (p, shift) = (Point { x: -1920, y: 1079 }, Scan::new(0x2A));
    let acts = [
        PointerAct::KeyDown(shift),
        PointerAct::Move(p),
        PointerAct::Down(Button::Right, p),
        PointerAct::Up(Button::Right, p),
        PointerAct::KeyUp(shift),
    ];
    let mouse = |press| Input::Mouse {
        dx: 0,
        dy: 65535,
        press,
        tag: REPLAY,
    };
    let want = [
        Input::Key {
            event: Key::Scan(shift).event(false),
            tag: REPLAY,
        },
        mouse(None),
        mouse(Some((Button::Right, false))),
        mouse(Some((Button::Right, true))),
        Input::Key {
            event: Key::Scan(shift).event(true),
            tag: REPLAY,
        },
    ];
    assert_eq!(batch(&acts, desk), want);
}

#[cfg(windows)]
#[test]
fn the_message_tables_match_windows() {
    use crate::hook::accept::user_key;
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
        WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };
    let pairs = [
        (WM_MOUSEMOVE, Kind::Move),
        (WM_LBUTTONDOWN, Kind::Down(Button::Left)),
        (WM_LBUTTONUP, Kind::Up(Button::Left)),
        (WM_RBUTTONDOWN, Kind::Down(Button::Right)),
        (WM_RBUTTONUP, Kind::Up(Button::Right)),
        (WM_MBUTTONDOWN, Kind::Down(Button::Middle)),
        (WM_MBUTTONUP, Kind::Up(Button::Middle)),
    ];
    for (msg, want) in pairs {
        assert_eq!(kind(msg), Some(want), "{msg:#x}");
    }
    assert_eq!(kind(WM_MOUSEWHEEL), None);
    let vk = u32::from(VK_ESCAPE.0);
    assert!(esc(Source::Physical, WM_KEYDOWN, vk, false, 0));
    assert!(esc(Source::Physical, WM_SYSKEYDOWN, vk, false, 0));
    let esc_key = Scan::new(0x01);
    let up = Some(PointerAct::KeyUp(esc_key));
    assert_eq!(user_key(Source::Physical, WM_KEYUP, esc_key, false, 0), up);
    assert_eq!(
        user_key(Source::Physical, WM_SYSKEYUP, esc_key, false, 0),
        up
    );
}
