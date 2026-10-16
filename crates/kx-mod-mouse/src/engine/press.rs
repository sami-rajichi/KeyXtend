//! A left press outside our windows: held back in Right-click mode until it turns into a click, a drag or a
//! right-click; picked up by Grab; or sent inside its modifiers.

use super::{Engine, Mode, Note, State, behind};
use kx_module_api::Mono;
use kx_module_api::ports::{BASE_DPI, Button, Point, PointerAct, PointerEvent, physical};

impl Engine {
    /// Nothing held back: a left press outside our windows starts a hold, a pick-up or a click with modifiers.
    pub(super) fn idle(
        &mut self,
        event: PointerEvent,
        now: Mono,
        acts: &mut Vec<PointerAct>,
    ) -> bool {
        match event {
            PointerEvent::Down {
                button: Button::Left,
                at,
                ours: false,
                dpi,
            } => self.press(at, dpi, now, acts),
            PointerEvent::Up {
                button: Button::Left,
                at,
            } if self.mods_down => {
                self.click_up(at, acts);
                true
            }
            _ => false,
        }
    }

    /// A left press outside at `at` on a screen of `dpi`; it never waits in Grab.
    fn press(&mut self, at: Point, dpi: u32, now: Mono, acts: &mut Vec<PointerAct>) -> bool {
        // If the hook missed the last click's release, its modifiers go up before this click takes its own.
        self.mods_up(acts);
        self.snap_mods();
        match self.mode {
            Mode::Grab { .. } => return self.pick(at, acts),
            Mode::Off => return self.press_through(at, acts),
            Mode::RightClick => {}
        }
        // A screen that reports no DPI counts as 100 %.
        let dpi = if dpi == 0 { BASE_DPI } else { dpi };
        let r = physical(f64::from(self.still_px), dpi);
        self.state = State::Pending { t0: now, p0: at, r };
        self.note(Note::HoldStart { at });
        true
    }

    /// A press is held back at `p0` with radius `r`, and its time has not come.
    pub(super) fn pending(
        &mut self,
        event: PointerEvent,
        p0: Point,
        r: i32,
        now: Mono,
        acts: &mut Vec<PointerAct>,
    ) -> bool {
        match event {
            PointerEvent::Up {
                button: Button::Left,
                ..
            } => {
                self.click_at(p0, acts);
                true
            }
            PointerEvent::Down {
                button: Button::Left,
                ..
            } => {
                // A second left press shows the hook missed the release: the held click goes first, then this press.
                self.click_at(p0, acts);
                let dropped = self.idle(event, now, acts);
                behind(event, dropped, acts)
            }
            PointerEvent::Move { at } if far(p0, at, r) => {
                self.replay(p0, acts);
                acts.push(PointerAct::Move(at));
                true
            }
            PointerEvent::Down { button, at, .. } => {
                self.replay(p0, acts);
                acts.push(PointerAct::Down(button, at));
                true
            }
            _ => false,
        }
    }

    /// The held press was a click: it goes through at `p0` inside its modifiers.
    fn click_at(&mut self, p0: Point, acts: &mut Vec<PointerAct>) {
        self.replay(p0, acts);
        acts.push(PointerAct::Up(Button::Left, p0));
        self.mods_up(acts);
    }

    /// Lets the held press through at `p0` inside its modifiers; the hold ends.
    pub(super) fn replay(&mut self, p0: Point, acts: &mut Vec<PointerAct>) {
        self.mods_down(acts);
        acts.push(PointerAct::Down(Button::Left, p0));
        self.state = State::Idle;
        self.note(Note::HoldEnd);
    }

    /// Fires a hold whose time has come at `now`: a right-click at its point, inside its modifiers. True when it
    /// fired.
    pub(super) fn fire_due(&mut self, now: Mono, acts: &mut Vec<PointerAct>) -> bool {
        let State::Pending { t0, p0, .. } = self.state else {
            return false;
        };
        if now < self.due(t0) {
            return false;
        }
        self.mods_down(acts);
        acts.extend([
            PointerAct::Down(Button::Right, p0),
            PointerAct::Up(Button::Right, p0),
        ]);
        self.state = State::AwaitUp;
        self.note(Note::Fired { at: p0 });
        self.mods_up(acts);
        true
    }
}

/// True when `to` is farther than `r` pixels from `p0`.
fn far(p0: Point, to: Point, r: i32) -> bool {
    let dx = i64::from(to.x) - i64::from(p0.x);
    let dy = i64::from(to.y) - i64::from(p0.y);
    let r = i64::from(r);
    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy)) > r.saturating_mul(r)
}
