//! Whose input reaches the engine, and hook messages as engine events (ADR-0027 decision 3).

use kx_module_api::ports::{Button, Point, PointerAct, Scan};

use super::Source;
use crate::keys::{EXTENDED_PREFIXES, REPORTED_EXTENDED, TAG, is_replay};

/// What a mouse message does.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A button went down.
    Down(Button),
    /// A button went up.
    Up(Button),
    /// The pointer moved.
    Move,
}

/// The mouse messages the engine hears (`WM_MOUSEMOVE`, then each button's down and up); wheels and the side
/// buttons pass untouched.
const MOUSE: [(u32, Kind); 7] = [
    (0x0200, Kind::Move),
    (0x0201, Kind::Down(Button::Left)),
    (0x0202, Kind::Up(Button::Left)),
    (0x0204, Kind::Down(Button::Right)),
    (0x0205, Kind::Up(Button::Right)),
    (0x0207, Kind::Down(Button::Middle)),
    (0x0208, Kind::Up(Button::Middle)),
];
/// `WM_KEYDOWN` and `WM_SYSKEYDOWN`: a key press, plain or with Alt held.
const KEY_DOWNS: [u32; 2] = [0x0100, 0x0104];
/// `WM_KEYUP` and `WM_SYSKEYUP`: a key let go, plain or with Alt held.
const KEY_UPS: [u32; 2] = [0x0101, 0x0105];
/// `VK_ESCAPE`.
const VK_ESCAPE: u32 = 0x1B;

/// What the mouse hook does with an event.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Take {
    /// Our own input came back: it passes, and the wait for it ends.
    Echo,
    /// Another program's input, or input the source ignores: it passes untouched.
    Pass,
    /// The engine decides it.
    Engine,
}

/// What happens to a mouse event with these marks. Ours (`TAG`, or `REPLAY` with any id) is an echo; the engine hears
/// only the hand, or in simulated mode only the gate tool's `SIM_TAG`.
pub(crate) fn take(source: Source, injected: bool, extra: usize) -> Take {
    if extra == TAG || is_replay(extra) {
        return Take::Echo;
    }
    if heard(source, injected, extra) {
        Take::Engine
    } else {
        Take::Pass
    }
}

/// True for an Esc press the engine hears: the hand's or our keyboard's (`TAG`), or in simulated mode the gate
/// tool's. Esc always passes on to the app, heard or not.
pub(crate) fn esc(source: Source, msg: u32, vk: u32, injected: bool, extra: usize) -> bool {
    let press = KEY_DOWNS.contains(&msg) && vk == VK_ESCAPE;
    let ours = source == Source::Physical && extra == TAG;
    press && (ours || heard(source, injected, extra))
}

/// A mouse event of `kind` at `at` as an act.
pub(crate) fn act(kind: Kind, at: Point) -> PointerAct {
    match kind {
        Kind::Down(button) => PointerAct::Down(button, at),
        Kind::Up(button) => PointerAct::Up(button, at),
        Kind::Move => PointerAct::Move(at),
    }
}

/// Key message `msg` for `scan` as an act; `None` for a message that is neither a press nor a release.
pub(crate) fn key_act(msg: u32, scan: Scan) -> Option<PointerAct> {
    if KEY_DOWNS.contains(&msg) {
        Some(PointerAct::KeyDown(scan))
    } else {
        KEY_UPS.contains(&msg).then_some(PointerAct::KeyUp(scan))
    }
}

/// The press or release of a key by the user in `source`, the hand or in simulated mode the gate tool, as an act.
pub(crate) fn user_key(
    source: Source,
    msg: u32,
    scan: Scan,
    injected: bool,
    extra: usize,
) -> Option<PointerAct> {
    key_act(msg, scan).filter(|_| heard(source, injected, extra))
}

/// The key a key hook event names, as we send it: the low byte of `code`, with the `E0` prefix when the event is
/// `extended`, unless Windows only reports that plain key as extended (right Shift).
pub(crate) fn sent_scan(code: u32, extended: bool) -> Scan {
    let [low, ..] = code.to_le_bytes();
    let plain = u16::from(low);
    if !extended || REPORTED_EXTENDED.contains(&plain) {
        Scan::new(plain)
    } else {
        Scan::new(u16::from_be_bytes([EXTENDED_PREFIXES[0], low]))
    }
}

/// True when input with these marks stands for the user in `source`.
fn heard(source: Source, injected: bool, extra: usize) -> bool {
    match sim_tag(source) {
        None => !injected,
        Some(tag) => injected && extra == tag,
    }
}

/// The tag the user's input carries in `source`; the hand's input carries none.
const fn sim_tag(source: Source) -> Option<usize> {
    match source {
        Source::Physical => None,
        #[cfg(any(test, feature = "sim-input"))]
        Source::Simulated => Some(crate::keys::SIM_TAG),
    }
}

/// What mouse message `msg` does; `None` for messages the engine does not hear.
pub(crate) fn kind(msg: u32) -> Option<Kind> {
    MOUSE.iter().find(|(m, _)| *m == msg).map(|&(_, k)| k)
}
