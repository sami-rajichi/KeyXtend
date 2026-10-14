//! G6's stand-in face and the pointer's track: the face sits over the document's lower part, as the keyboard sits
//! below a document, and the track keeps the last point outside it, as P5's tracker will.
#![cfg(windows)]

use spike_core::hold::Pt;
use spike_core::uia;
use windows::Win32::Foundation::RECT;

use crate::probecfg::G6;
use crate::{simin, simuser};

/// The pointer's last point outside the face.
#[derive(Debug, Clone, Copy)]
pub struct Track {
    face: RECT,
    last: Option<Pt>,
}

impl Track {
    /// A track that has seen no point outside `face` yet.
    pub fn new(face: RECT) -> Self {
        Self { face, last: None }
    }

    /// Takes the pointer at `p`; a point inside the face is ignored.
    pub fn see(&mut self, p: Pt) {
        if !simin::inside(&self.face, p) {
            self.last = Some(p);
        }
    }

    /// The last point seen outside the face, if any.
    pub fn last(&self) -> Option<Pt> {
        self.last
    }
}

/// The stand-in face for document window `doc` on a screen with work area `work`: a `face_px` square centred under the
/// document, its top `face_overlap_px` above the window's lower edge, kept inside the work area.
pub fn face(doc: &RECT, work: &RECT, g: &G6) -> RECT {
    let size = g.face_px;
    let left = (uia::centre(doc).x - size / 2)
        .min(work.right - size)
        .max(work.left);
    let top = (doc.bottom - g.face_overlap_px)
        .min(work.bottom - size)
        .max(work.top);
    RECT {
        left,
        top,
        right: left + size,
        bottom: top + size,
    }
}

/// The glide's points: `from`, then `steps` even steps to rest at the face's centre.
pub fn glide(from: Pt, face: &RECT, steps: i32) -> Vec<Pt> {
    let rest = uia::centre(face);
    std::iter::once(from)
        .chain(simuser::path(from, rest, steps))
        .collect()
}

/// Point `p` as its offset from the top-left of window box `doc`, never as a screen point.
pub fn offset(doc: &RECT, p: Pt) -> [i32; 2] {
    [p.x - doc.left, p.y - doc.top]
}

#[cfg(test)]
mod tests;
