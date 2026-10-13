//! What our own sent input holds down (ledger ruling 21), so the host lets go itself when the engine cannot. The record
//! is fixed in size and never allocates.

use kx_module_api::ports::{Button, Point, PointerAct, Scan};

/// Keys our input can hold at once: the engine's latched modifiers (eight in `kx-mod-mouse`), with as many again spare.
pub(crate) const HELD_KEYS: usize = 16;
/// One slot per `Button`; `slot` keeps every button below it.
pub(crate) const SLOTS: usize = 3;

/// The slot of `button`. The match is exhaustive, so a new button fails to compile until it has a slot.
pub(crate) const fn slot(button: Button) -> usize {
    match button {
        Button::Left => 0,
        Button::Right => 1,
        Button::Middle => 2,
    }
}

/// The buttons and keys our sent input pressed and has not let go.
#[derive(Debug, Default)]
pub(crate) struct Held {
    /// Per slot, the button and where it went down.
    buttons: [Option<(Button, Point)>; SLOTS],
    /// The keys held, packed in the order they went down; a key past the room is not kept.
    keys: [Option<Scan>; HELD_KEYS],
}

impl Held {
    /// Notes what `acts`, all taken by the OS, pressed and let go.
    pub(crate) fn sent(&mut self, acts: &[PointerAct]) {
        for &act in acts {
            match act {
                PointerAct::Down(button, at) => self.set(button, Some((button, at))),
                PointerAct::Up(button, _) => self.set(button, None),
                PointerAct::KeyDown(scan) => self.press(scan),
                PointerAct::KeyUp(scan) => self.lift(scan),
                PointerAct::Move(_) => {}
            }
        }
    }

    /// An Up for each button still held, at `at` or else where it went down, then a key-up for each key, newest first.
    pub(crate) fn release(&self, at: Option<Point>) -> Vec<PointerAct> {
        let buttons = self.buttons.iter().flatten();
        let ups = buttons.map(|&(button, down)| PointerAct::Up(button, at.unwrap_or(down)));
        let keys = self
            .keys
            .iter()
            .rev()
            .flatten()
            .map(|&scan| PointerAct::KeyUp(scan));
        ups.chain(keys).collect()
    }

    /// Notes `button` down at a point, or up.
    fn set(&mut self, button: Button, state: Option<(Button, Point)>) {
        if let Some(cell) = self.buttons.get_mut(slot(button)) {
            *cell = state;
        }
    }

    /// Notes `scan` held once, after the others; a key past the room is not kept.
    fn press(&mut self, scan: Scan) {
        if self.keys.contains(&Some(scan)) {
            return;
        }
        if let Some(free) = self.keys.iter_mut().find(|k| k.is_none()) {
            *free = Some(scan);
        }
    }

    /// Lets `scan` go and closes its gap, so the keys stay in the order they went down.
    fn lift(&mut self, scan: Scan) {
        let Some(at) = self.keys.iter().position(|k| *k == Some(scan)) else {
            return;
        };
        if let Some(tail) = self.keys.get_mut(at..) {
            tail.rotate_left(1);
        }
        if let Some(last) = self.keys.last_mut() {
            *last = None;
        }
    }
}
