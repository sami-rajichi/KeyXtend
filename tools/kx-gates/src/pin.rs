//! A window pinned for the scroll pad, as G6, G8, G9 and G10 lay one: under the keyboard, with the pad found on it and
//! the glide's start above the keyboard on the window; dropping it puts the window back.
#![cfg(windows)]

use kx_mod_keyboard::view::DPAD_NAME;
use spike_core::pt::Pt;
use windows::Win32::Foundation::{HWND, POINT, RECT};

use crate::aim::{self, Placed};
use crate::apps::{Ctx, Opened};
use crate::keymap::Reader;
use crate::padgeom::{Pad, Shape};
use crate::simuser::guard;
use crate::win::{self, sleep_ms};

/// A window laid under the keyboard for the pad.
pub struct Pinned {
    /// The pad on screen.
    pub pad: Pad,
    /// The keyboard window's box.
    pub kb: RECT,
    /// The laid window's box.
    pub win: RECT,
    /// Where the glide onto the pad starts, on the window.
    pub start: POINT,
    _placed: Placed,
}

/// `p` as the simulated user's point.
pub fn pt(p: POINT) -> Pt {
    Pt { x: p.x, y: p.y }
}

impl Pinned {
    /// Lays `app` under keyboard window `kb` as `[pad]` asks, once the pad shows there, and finds the glide's start.
    pub fn new(ctx: &Ctx, kb: HWND, app: &Opened) -> Result<Self, String> {
        let kb_box = win::rect(kb)?;
        let pad_box = Reader::new()?.rect(kb, DPAD_NAME)?;
        let pad = Pad::of(pad_box, Shape::shipped()?).ok_or("the scroll pad has no box")?;
        let (x, reach) = (pad.centre().x, ctx.cfg.pad.reach());
        let placed = Placed::lay(app.hwnd, kb_box, x, reach)?;
        sleep_ms(ctx.cfg.probes.settle_ms);
        let win_box = win::rect(app.hwnd)?;
        let start = aim::start(win_box, kb_box, x, reach.lead_px).ok_or("no room above the pad")?;
        guard(app, &[pt(start)])?;
        Ok(Self {
            pad,
            kb: kb_box,
            win: win_box,
            start,
            _placed: placed,
        })
    }

    /// Glides from the start onto the pad's hub with `mover`, then waits the pad's settle; the glide's last point
    /// outside the keyboard, where the pad scrolls.
    pub fn glide_in(
        &self,
        ctx: &Ctx,
        mover: impl Fn(POINT) -> Result<(), String>,
    ) -> Result<POINT, String> {
        let points = aim::glide(self.start, self.pad.centre(), ctx.cfg.probes.move_steps);
        aim::walk(&points, ctx.cfg.sim.step_ms, mover)?;
        sleep_ms(ctx.cfg.pad.settle_ms);
        aim::last_outside(&points, self.kb)
            .ok_or_else(|| "the glide never left the keyboard".to_string())
    }
}
