//! Crop's frame (plan 2026-11-26 "Crop"): after Crop's second click the part shows framed, a click on a handle picks
//! it up and the next sets it by the frame's rules, ✓ cuts to it as one step, and ✗, Esc, Undo or another tool drop it.
//! While a handle is held, ✓ and ✗ wait and Esc lets go of the handle only.

use kx_module_api::ports::{Crop, CropFrame, CropKey, Frame, Grip, Handle, Point, Rect, Scale};

use super::history::Step;
use super::{Canvas, Did, hit};
use crate::frame;

/// Crop's frame waiting: the part it would keep and the handle held.
#[derive(Copy, Clone, Debug)]
pub(super) struct Waiting {
    part: Rect,
    grip: Option<Grip>,
}

impl Canvas {
    /// The frame waiting, within the part kept so far.
    pub(super) fn frame(&self) -> Option<CropFrame> {
        let waiting = self.waiting?;
        let frame = Frame {
            within: self.sheet.crop.rect(),
            min: self.min,
            grip: waiting.grip,
        };
        Some(CropFrame {
            part: waiting.part,
            frame,
        })
    }

    /// Crop's second click: `part`, inside what shows, waits framed for ✓ or ✗ and cuts nothing yet.
    pub(super) fn wait(&mut self, part: Rect) -> Did {
        self.waiting = Some(Waiting { part, grip: None });
        Did::Show
    }

    /// A click at `at` while the frame waits, shown at `scale`: it sets the handle held, else picks up the handle
    /// within reach; anywhere else it does nothing.
    pub(super) fn adjust(&mut self, at: Point, scale: Scale) -> Did {
        let (within, min) = (self.sheet.crop.rect(), self.min);
        let reach = hit::reach(&self.fit, scale);
        let Some(waiting) = self.waiting.as_mut() else {
            return Did::Stay;
        };
        if let Some(grip) = waiting.grip.take() {
            if let Some(part) = frame::moved(waiting.part, grip, at, within, min) {
                waiting.part = part;
            }
            return Did::Show;
        }
        match frame::hit(waiting.part, at, reach) {
            Some(handle) => {
                waiting.grip = Some(Grip { handle, at });
                Did::Show
            }
            None => Did::Stay,
        }
    }

    /// A reader's press on `handle`: it is picked up at its spot, a held one dropped; on the held one, or with no frame
    /// waiting, nothing.
    pub fn handle(&mut self, handle: Handle) -> Did {
        let Some(waiting) = self.waiting.as_mut() else {
            return Did::Stay;
        };
        if waiting.grip.is_some_and(|g| g.handle == handle) {
            return Did::Stay;
        }
        match frame::spot(waiting.part, handle) {
            Some(at) => {
                waiting.grip = Some(Grip { handle, at });
                Did::Show
            }
            None => Did::Stay,
        }
    }

    /// ✓ cuts to the frame as last set, as one step, unless it frames the part already kept; ✗ drops the frame. Either
    /// does nothing with no frame waiting, or while a handle is held, so an overshooting set never acts.
    pub fn crop(&mut self, key: CropKey) -> Did {
        let Some(waiting) = self.waiting.take_if(|w| w.grip.is_none()) else {
            return Did::Stay;
        };
        let kept = Crop::new(waiting.part, self.width, self.height);
        match (key, kept) {
            (CropKey::Ok, Some(crop)) if crop != self.sheet.crop => self.apply(Step::Crop(crop)),
            _ => Did::Show,
        }
    }

    /// Our keyboard's Esc lets go of a held handle, else drops the frame waiting (Region's edge 4); with none it does
    /// nothing, so the flow takes the Esc.
    pub fn esc(&mut self) -> Did {
        let Some(waiting) = self.waiting.as_mut() else {
            return Did::Stay;
        };
        if waiting.grip.take().is_none() {
            self.waiting = None;
        }
        Did::Show
    }
}
