//! The engine's acts as one batch of input: mouse acts at absolute virtual-desktop points, keys by scan code, and
//! every event tagged `REPLAY` with its own id. `win` turns each one into a Windows `INPUT`.

use std::fmt;

use kx_module_api::Redacted;
use kx_module_api::ports::{Button, Point, PointerAct};

use crate::keys::replay_tag;
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

/// One input of a batch, before `win` makes it a Windows `INPUT`; its `Debug` hides each point and key.
#[derive(Copy, Clone, PartialEq, Eq)]
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

/// Shows the button and tag only: a point can show what was clicked, and a key what was typed.
impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Mouse { dx, dy, press, tag } => f
                .debug_struct("Mouse")
                .field("dx", &Redacted::new(dx))
                .field("dy", &Redacted::new(dy))
                .field("press", &press)
                .field("tag", &tag)
                .finish(),
            Self::Key { event, tag } => f
                .debug_struct("Key")
                .field("event", &Redacted::new(event))
                .field("tag", &tag)
                .finish(),
        }
    }
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

/// `acts` as one batch for the virtual desktop `desk`, each event tagged `REPLAY` with its id, counted from `first`.
pub(crate) fn batch(acts: &[PointerAct], desk: Desk, first: u32) -> Vec<Input> {
    let mouse = |p: Point, press, tag| Input::Mouse {
        dx: normalise(p.x, desk.left, desk.width),
        dy: normalise(p.y, desk.top, desk.height),
        press,
        tag,
    };
    let key = |scan, up, tag| Input::Key {
        event: Key::Scan(scan).event(up),
        tag,
    };
    let ids = std::iter::successors(Some(first), |id| Some(id.wrapping_add(1)));
    acts.iter()
        .zip(ids.map(replay_tag))
        .map(|(&act, tag)| match act {
            PointerAct::Move(p) => mouse(p, None, tag),
            PointerAct::Down(b, p) => mouse(p, Some((b, false)), tag),
            PointerAct::Up(b, p) => mouse(p, Some((b, true)), tag),
            PointerAct::KeyDown(scan) => key(scan, false, tag),
            PointerAct::KeyUp(scan) => key(scan, true, tag),
        })
        .collect()
}
