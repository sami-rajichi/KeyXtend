//! A region's frame on the frozen screens (plan 2026-11-26 "Region"): a click on a handle, or a reader's press on it,
//! picks it up and the next click sets it, cutting the part again; a click away from the frame starts a new region
//! there, and Esc drops a held handle first.

use kx_module_api::ports::{BarKey, FrozenScreen, Grip, Handle, Part, Point, physical};

use super::{Flow, Held, Picking, Stage, Strip};
use crate::{frame, part};

/// The way a region's strip was picked, which a click away picks again: a video's before it records, else a picture's.
const fn way(held: &Held) -> BarKey {
    if matches!(held, Held::Ready(_)) {
        BarKey::VideoRegion
    } else {
        BarKey::Region
    }
}

/// `strip` showing `part` with its screen found again, and a picture's pixels cut again; a part on no screen keeps the
/// old one.
fn recut(strip: &mut Strip, part: Part, screens: &[FrozenScreen]) {
    let Some(home) = part::home(part, screens) else {
        return;
    };
    if let Held::Picture(shot) = &mut strip.held {
        let Some(cut) = part.cut(screens) else {
            return;
        };
        *shot = cut;
    }
    (strip.part, strip.home) = (part, home);
}

impl Flow {
    /// A click on the frozen screens while a region's strip shows: it sets the handle held, picks one up, or, away from
    /// the frame, starts a region there. Nothing acts while a Copy or Save waits, the question shows or a tick ends.
    pub(super) fn adjust(&mut self, at: Point) {
        if !self.framing() {
            return;
        }
        let reach = self.reach_at(at);
        let (Stage::Strip(strip), Some(frozen)) = (&mut self.stage, &self.frozen) else {
            return;
        };
        let way = way(&strip.held);
        let Some(frame) = strip.frame.as_mut() else {
            return;
        };
        if let Some(grip) = frame.grip.take() {
            let moved = frame::moved(strip.part.rect(), grip, at, frame.within, frame.min);
            if let Some(part) = moved.and_then(|r| part::framed(way, r, &frozen.screens)) {
                recut(strip, part, &frozen.screens);
            }
            return;
        }
        match frame::hit(strip.part.rect(), at, reach) {
            Some(handle) => frame.grip = Some(Grip { handle, at }),
            None if !part::near(strip.part, at, reach) => {
                let corner = Some(at);
                self.stage = Stage::Picking(Picking { way, corner });
            }
            None => {}
        }
    }

    /// A reader's press on `handle`: it is picked up at its spot, a held one dropped; held already, it stays as it is.
    pub(super) fn pick_up(&mut self, handle: Handle) {
        if !self.framing() {
            return;
        }
        let Stage::Strip(Strip {
            part,
            frame: Some(frame),
            ..
        }) = &mut self.stage
        else {
            return;
        };
        if frame.grip.is_some_and(|g| g.handle == handle) {
            return;
        }
        if let Some(at) = frame::spot(part.rect(), handle) {
            frame.grip = Some(Grip { handle, at });
        }
    }

    /// No Copy or Save waits, no question shows and no tick ends the snip, so the frame takes clicks.
    fn framing(&self) -> bool {
        let ending = self.due.is_some_and(|due| due.ends);
        self.free() && !ending
    }

    /// Drops a held handle, the frame back as before its pick-up; true when one was held.
    pub(super) fn let_go(&mut self) -> bool {
        match &mut self.stage {
            Stage::Strip(Strip {
                frame: Some(frame), ..
            }) => frame.grip.take().is_some(),
            _ => false,
        }
    }

    /// Half the theme's handle click area, in physical pixels on the screen holding `at`; 0 between screens.
    fn reach_at(&self, at: Point) -> i32 {
        let screens = self.frozen.as_ref().map_or(&[][..], |f| &f.screens);
        let half = f64::from(self.hit_px) / frame::SIDES;
        part::screen(at, screens).map_or(0, |s| physical(half, s.dpi()))
    }
}
