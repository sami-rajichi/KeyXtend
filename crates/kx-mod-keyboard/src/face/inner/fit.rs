//! The keyboard's size and place on its screen: it always fits the screen it is on, so the size buttons at the top
//! bar's right never leave it, and only a move by the user is saved.

use super::{Inner, STATE, notice};
use crate::place::{Extent, landing, room, screen_of};
use crate::sizer::{SizeError, Sizer};
use kx_module_api::keys;
use kx_module_api::ports::{BASE_DPI, FaceError, FaceReply, Point, Rect, Screen, physical};

impl Inner {
    /// The window at size 1, in logical pixels: the plate and the resize corner hanging past it.
    fn unit(&self) -> Extent {
        Extent {
            plate: (self.plate.w, self.plate.h),
            overhang: self.overhang,
        }
    }

    /// The window now, in logical pixels.
    pub(super) fn size(&self) -> Extent {
        self.unit().scale(self.sizer.size())
    }

    /// The plate's size in the physical pixels of the screen the window is on, as the pointer counts them.
    pub(super) fn plate_px(&self) -> (f64, f64) {
        let size = self.size();
        let on = self
            .placer
            .at()
            .and_then(|at| screen_of(at, size.whole(), &self.screens));
        let dpi = on.map_or(BASE_DPI, |s| s.dpi);
        let (w, h) = size.plate;
        (f64::from(physical(w, dpi)), f64::from(physical(h, dpi)))
    }

    /// The window ended at `rect`, its corner hanging `overhang` past the plate at size 1: where the face put it, the
    /// saved place is fitted to `screens` again without saving; anywhere else the user moved it, so it is saved.
    pub(super) fn placed(&mut self, rect: Rect, screens: Vec<Screen>, overhang: f64) -> FaceReply {
        self.screens = screens;
        self.overhang = if overhang.is_finite() {
            overhang.max(0.0)
        } else {
            0.0
        };
        let to = Point {
            x: rect.x,
            y: rect.y,
        };
        let moved = self.placer.at().is_some_and(|at| at != to);
        let (board, saved) = if moved {
            let resized = self.fit_size(Some(to));
            let size = self.size();
            let saved = self.placer.moved(to, size, &self.screens).map(|_| ());
            (self.settle() || resized, saved)
        } else {
            (self.start_place(), Ok(()))
        };
        let reply = FaceReply {
            board,
            state: self.placer.at() != Some(to),
            notice: None,
        };
        match saved {
            Ok(()) => reply,
            Err(_) => notice(keys::SETTINGS_UNSAVED, FaceReply { board, ..STATE }),
        }
    }

    /// Puts the keyboard where it starts, fitted to its screen and saving nothing; true when the size changed.
    pub(super) fn start_place(&mut self) -> bool {
        let resized = self.fit_size(self.placer.saved());
        let size = self.size();
        self.placer.start(size, &self.screens);
        self.settle() || resized
    }

    /// Fits the size to the screen the keyboard is on, then keeps it inside that screen; true when the size changed.
    fn settle(&mut self) -> bool {
        let resized = self.fit_size(self.placer.at());
        let size = self.size();
        self.placer.keep(size, &self.screens);
        resized
    }

    /// Caps the size at the room on the screen a window at `at` lands on, home when `None`; true when it changed.
    fn fit_size(&mut self, at: Option<Point>) -> bool {
        let on = landing(at, self.size().whole(), &self.screens);
        on.and_then(|s| room(self.unit().whole(), s))
            .is_some_and(|fits| self.sizer.fit(fits))
    }

    /// Applies a size change and keeps the keyboard on its screen, fitted to the screen it now lands on: the board
    /// redraws when the size changed, the state when it moved, with a notice when the size could not be saved.
    pub(super) fn resized(
        &mut self,
        change: impl FnOnce(&mut Sizer) -> Result<bool, SizeError>,
    ) -> Result<FaceReply, FaceError> {
        let before = self.placer.at();
        let changed = change(&mut self.sizer);
        let size = self.size();
        self.placer.keep(size, &self.screens);
        let refit = self.settle();
        let state = self.placer.at() != before;
        let redraw = |board: bool| FaceReply {
            board: board || refit,
            state,
            notice: None,
        };
        changed.map(redraw).or_else(|e| match e {
            SizeError::NoPreset(index) => Err(FaceError::NoChoice(index)),
            SizeError::Unsaved(_) => Ok(notice(keys::SETTINGS_UNSAVED, redraw(true))),
        })
    }
}
