//! What our own sent input holds down (ledger rulings 21 and 22), so the host lets go itself when the engine cannot.
//! The record is fixed in size, so noting input and letting go never allocate.

use kx_module_api::ports::{Button, MAX_HELD_KEYS, Point, PointerAct, Scan};

/// One slot per `Button`; each arm of `slot` checks at build time that its slot is below it.
pub(crate) const SLOTS: usize = 3;
/// The most acts a let-go adds: an Up per button and a key-up per key.
pub(crate) const LET_GO: usize = SLOTS + MAX_HELD_KEYS;

/// The slot of `button`: its own discriminant, so slots never repeat. The match is exhaustive and every arm goes
/// through `below_slots` at build time, so a new button cannot build until its slot is below `SLOTS`.
pub(crate) const fn slot(button: Button) -> usize {
    match button {
        Button::Left => const { below_slots(Button::Left as usize) },
        Button::Right => const { below_slots(Button::Right as usize) },
        Button::Middle => const { below_slots(Button::Middle as usize) },
    }
}

/// `slot` as given, or a failed build when it is not below `SLOTS`.
const fn below_slots(slot: usize) -> usize {
    assert!(slot < SLOTS, "every button needs a slot below SLOTS");
    slot
}

/// The buttons and keys our sent input pressed and has not let go.
#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Held {
    /// Per slot, the button and where it went down.
    buttons: [Option<(Button, Point)>; SLOTS],
    /// The keys held, packed in the order they went down; a key past the room is not kept.
    keys: [Option<Scan>; MAX_HELD_KEYS],
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

    /// The press of ours, held here, that `release` lets go: a Down for an Up, a key press for a key-up.
    pub(crate) fn holds(&self, release: PointerAct) -> Option<PointerAct> {
        match release {
            PointerAct::Up(button, _) => {
                let (button, at) = self.buttons.get(slot(button)).copied().flatten()?;
                Some(PointerAct::Down(button, at))
            }
            PointerAct::KeyUp(scan) => self
                .keys
                .contains(&Some(scan))
                .then_some(PointerAct::KeyDown(scan)),
            _ => None,
        }
    }

    /// Lets go of what `release`, an Up or a key-up, lets go, without sending it: the hand let it go itself.
    pub(crate) fn forget(&mut self, release: PointerAct) {
        if matches!(release, PointerAct::Up(..) | PointerAct::KeyUp(_)) {
            self.sent(&[release]);
        }
    }

    /// An Up for each button still held, at `at` or else where it went down, then a key-up for each key, newest first.
    pub(crate) fn release(&self, at: Option<Point>) -> impl Iterator<Item = PointerAct> + '_ {
        let buttons = self.buttons.iter().flatten();
        let ups = buttons.map(move |&(button, down)| PointerAct::Up(button, at.unwrap_or(down)));
        let keys = self
            .keys
            .iter()
            .rev()
            .flatten()
            .map(|&scan| PointerAct::KeyUp(scan));
        ups.chain(keys)
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
