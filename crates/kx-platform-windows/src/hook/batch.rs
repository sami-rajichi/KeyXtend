//! The engine's acts as one batch of input: mouse acts at absolute virtual-desktop points, keys by scan code, and
//! every event tagged `REPLAY`. `win` turns each one into a Windows `INPUT`.

use kx_module_api::ports::{Button, Point, PointerAct};

use crate::keys::REPLAY;
use crate::strokes::{Key, KeyEvent};

/// The largest absolute mouse coordinate; the virtual desktop maps onto 0..=this.
const ABS_MAX: f64 = 65535.0;

/// The virtual desktop, in physical pixels.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Desk {
    /// Its left edge.
    pub left: i32,
    /// Its top edge.
    pub top: i32,
    /// Its width.
    pub width: i32,
    /// Its height.
    pub height: i32,
}

/// One input of a batch, before `win` makes it a Windows `INPUT`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Input {
    /// The pointer goes to absolute (`dx`, `dy`), with a button pressed there, or released when its flag is set.
    Mouse {
        /// The absolute x, 0..=65535 across the virtual desktop.
        dx: i32,
        /// The absolute y.
        dy: i32,
        /// The button and whether it goes up.
        press: Option<(Button, bool)>,
        /// The `dwExtraInfo` mark.
        tag: usize,
    },
    /// A key goes down or up by scan code.
    Key {
        /// The key event.
        event: KeyEvent,
        /// The `dwExtraInfo` mark.
        tag: usize,
    },
}

/// Maps pixel `v` over `start..start+size` to the 0..=65535 absolute range (P1's `normalise`).
#[allow(
    clippy::cast_possible_truncation,
    reason = "The result is rounded and lies within 0..=65535 for a pixel on the desktop."
)]
pub(crate) fn normalise(v: i32, start: i32, size: i32) -> i32 {
    let span = f64::from(size.saturating_sub(1).max(1));
    (f64::from(v.saturating_sub(start)) * ABS_MAX / span).round() as i32
}

/// `acts` as one batch for the virtual desktop `desk`, each event tagged `REPLAY`.
pub(crate) fn batch(acts: &[PointerAct], desk: Desk) -> Vec<Input> {
    let mouse = |p: Point, press| Input::Mouse {
        dx: normalise(p.x, desk.left, desk.width),
        dy: normalise(p.y, desk.top, desk.height),
        press,
        tag: REPLAY,
    };
    let key = |scan, up| Input::Key {
        event: Key::Scan(scan).event(up),
        tag: REPLAY,
    };
    acts.iter()
        .map(|&act| match act {
            PointerAct::Move(p) => mouse(p, None),
            PointerAct::Down(b, p) => mouse(p, Some((b, false))),
            PointerAct::Up(b, p) => mouse(p, Some((b, true))),
            PointerAct::KeyDown(scan) => key(scan, false),
            PointerAct::KeyUp(scan) => key(scan, true),
        })
        .collect()
}

/// True when `acts` hold mouse input, which comes back through the mouse hook.
pub(crate) fn has_mouse(acts: &[PointerAct]) -> bool {
    acts.iter().any(|act| {
        matches!(
            act,
            PointerAct::Move(_) | PointerAct::Down(..) | PointerAct::Up(..)
        )
    })
}
