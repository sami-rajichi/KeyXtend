//! G19's acts, each as the simulated user (`SIM_TAG`), whom the test build's engine hears as the hand, then a pause of
//! `settle_ms` unless a wait timed from the input follows. Clicks and drags go only where the gate's own window is on
//! top, and keys, ours included, only while it is in front.
#![cfg(windows)]

use std::cell::Cell;
use std::time::Instant;

use kx_platform_windows::keys::SIM_TAG;
use spike_core::pt::Pt;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::INPUT;

use crate::apps::{Ctx, Opened};
use crate::ownkeys::{self, Desk, Real};
use crate::product::Keyboard;
use crate::simin::{self, Act};
use crate::simuser::{self, guard, keys_ours};
use crate::win::sleep_ms;

/// `p` as the simulated user's point.
pub fn pt(p: POINT) -> Pt {
    Pt { x: p.x, y: p.y }
}

/// The presses of `vks` in order, then their releases in reverse, tagged `SIM_TAG`.
fn strokes(vks: &[u16]) -> Vec<INPUT> {
    let downs = vks
        .iter()
        .map(|&vk| crate::keys::tagged(vk, false, SIM_TAG));
    let ups = vks
        .iter()
        .rev()
        .map(|&vk| crate::keys::tagged(vk, true, SIM_TAG));
    downs.chain(ups).collect()
}

/// Runs `press`, a click on our keyboard's key, once `desk` shows `app` in front with our tab, as the key types there.
fn typed(
    desk: &dyn Desk,
    app: &Opened,
    press: &dyn Fn() -> Result<(), String>,
) -> Result<(), String> {
    ownkeys::click(desk, app, press)
}

/// What follows an act.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Then {
    /// A pause of `settle_ms`, so the app takes the act.
    Settle,
    /// No pause, so a wait timed from the input starts at once.
    Watch,
}

/// The ms to pause after an act followed by `then`, at `settle_ms`.
pub fn pause_ms(then: Then, settle_ms: u64) -> u64 {
    match then {
        Then::Settle => settle_ms,
        Then::Watch => 0,
    }
}

/// The gate's acts in one run.
pub struct Acts<'a> {
    ctx: &'a Ctx<'a>,
    sent: Cell<Instant>,
}

impl<'a> Acts<'a> {
    /// The acts of a run.
    pub fn new(ctx: &'a Ctx<'a>) -> Self {
        Self {
            ctx,
            sent: Cell::new(Instant::now()),
        }
    }

    /// When the last act's input ended, before its pause.
    pub fn sent(&self) -> Instant {
        self.sent.get()
    }

    /// The pause after each act, timed from the input's end.
    fn settle(&self) {
        self.after(Then::Settle);
    }

    /// Marks the input's end, then pauses as `then` says.
    fn after(&self, then: Then) {
        self.sent.set(Instant::now());
        sleep_ms(pause_ms(then, self.ctx.cfg.g19_product.settle_ms));
    }

    /// A short click at `p` on `app`'s window.
    pub fn click(&self, app: &Opened, p: POINT) -> Result<(), String> {
        guard(app, &[pt(p)])?;
        simuser::press(self.ctx, pt(p))?;
        self.settle();
        Ok(())
    }

    /// Two short clicks at `p` on `app`'s window, as one double click.
    pub fn double(&self, app: &Opened, p: POINT) -> Result<(), String> {
        guard(app, &[pt(p)])?;
        simuser::press(self.ctx, pt(p))?;
        simuser::press(self.ctx, pt(p))?;
        self.settle();
        Ok(())
    }

    /// The left button pressed at `p` on `app`'s window and held.
    pub fn down(&self, app: &Opened, p: POINT) -> Result<(), String> {
        guard(app, &[pt(p)])?;
        simin::as_user(&[Act::Move(pt(p)), Act::Down(pt(p))])?;
        self.settle();
        Ok(())
    }

    /// The held left button let go at `p`.
    pub fn up(&self, p: POINT) -> Result<(), String> {
        simin::as_user(&[Act::Up(pt(p))])?;
        self.settle();
        Ok(())
    }

    /// The pointer's moves from just after `from` to `to`.
    fn moves(&self, from: POINT, to: POINT) -> Result<(), String> {
        for at in simuser::path(pt(from), pt(to), self.ctx.cfg.probes.move_steps) {
            simin::as_user(&[Act::Move(at)])?;
            sleep_ms(self.ctx.cfg.sim.step_ms);
        }
        Ok(())
    }

    /// A drag from `from` to `to` on `app`'s window, the pointer gliding between.
    pub fn drag(&self, app: &Opened, from: POINT, to: POINT) -> Result<(), String> {
        guard(app, &[pt(from), pt(to)])?;
        simin::as_user(&[Act::Move(pt(from)), Act::Down(pt(from))])?;
        self.moves(from, to)?;
        simin::as_user(&[Act::Up(pt(to))])?;
        self.settle();
        Ok(())
    }

    /// The pointer glides from `from` to `to` on `app`'s window, with no button held, as Grab carries.
    pub fn glide(&self, app: &Opened, from: POINT, to: POINT) -> Result<(), String> {
        guard(app, &[pt(from), pt(to)])?;
        self.moves(from, to)?;
        self.settle();
        Ok(())
    }

    /// Keys `vks` pressed together while `app` is in front.
    pub fn keys(&self, app: &Opened, vks: &[u16]) -> Result<(), String> {
        self.keys_then(app, vks, Then::Settle)
    }

    /// Keys `vks` pressed together while `app` is in front, then what `then` says.
    pub fn keys_then(&self, app: &Opened, vks: &[u16], then: Then) -> Result<(), String> {
        keys_ours(app)?;
        crate::keys::send(&strokes(vks))?;
        self.after(then);
        Ok(())
    }

    /// A short click on our keyboard's key `id`, which types into `app`: only while `app` is in front with our tab,
    /// once the keyboard is on top there.
    pub fn key(&self, app: &Opened, id: &str) -> Result<(), String> {
        let press = || {
            let at = Keyboard::find()?.top_at(id)?;
            simuser::press(self.ctx, pt(at))
        };
        typed(&Real, app, &press)?;
        self.settle();
        Ok(())
    }

    /// A short click at `p`, once window `on` is on top there, as on the strip.
    pub fn press_on(&self, on: windows::Win32::Foundation::HWND, p: POINT) -> Result<(), String> {
        let top = crate::win::root_at(p);
        if top != on {
            return Err(format!(
                "{} covers the point to click",
                crate::win::describe(top)
            ));
        }
        simuser::press(self.ctx, pt(p))?;
        self.settle();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ownkeys::fake::{Fake, OUR_TAB, OWNER_TAB, hwnd, notepad};

    #[test]
    fn our_keyboards_keys_type_only_while_the_gates_window_shows_our_tab_in_front() {
        let app = notepad(None);
        let press = || Ok(());
        let ours = Fake::new(app.hwnd, &[OUR_TAB]);
        assert_eq!(typed(&ours, &app, &press), Ok(()));
        assert_eq!(ours.clicks.get(), 1);
        let away = Fake::new(hwnd(9), &[OUR_TAB]);
        let e = typed(&away, &app, &press).expect_err("another window in front");
        assert!(e.contains("held back") && away.quiet(), "{e}");
        let tab = Fake::new(app.hwnd, &[OWNER_TAB]);
        assert!(
            typed(&tab, &app, &press).is_err() && tab.quiet(),
            "the owner's tab"
        );
    }
}
