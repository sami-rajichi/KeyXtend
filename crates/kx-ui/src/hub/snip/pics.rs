//! The pictures Snip's provider serves: each frozen screen, then the editor's picture, shared and never copied, under
//! URLs that change only with their pixels; they go with the page that held them, so a snip's end lets them go. And a
//! flattened picture cut to the kept part.

use kx_module_api::ports::{BGRX_BYTES, Shot, SnipPage, SnipStage};
use std::sync::Arc;

/// The provider's name: QML draws `image://snip/<generation>/<place>`.
pub(crate) const SHOTS: &str = "snip";
/// What parts a picture id: the generation, then the picture's place.
const PART: char = '/';

/// The pictures kept, and the generation their URLs carry.
#[derive(Default)]
pub(super) struct Pics {
    generation: u64,
    shots: Vec<Shot>,
}

impl Pics {
    /// Keeps `page`'s pictures; the generation moves on only when their pixels changed, so QML loads nothing again.
    pub fn keep(&mut self, page: &SnipPage) {
        let shots = shots_of(page);
        let shared = |(a, b): (&Shot, &Shot)| Arc::ptr_eq(a.bgrx().expose(), b.bgrx().expose());
        let same = shots.len() == self.shots.len() && shots.iter().zip(&self.shots).all(shared);
        if !same {
            self.generation = self.generation.wrapping_add(1);
        }
        self.shots = shots;
    }

    /// The generation the URLs carry now.
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Picture `id`, `<generation>/<place>`, while that generation is kept; a clone shares its pixels.
    pub fn get(&self, id: &str) -> Option<Shot> {
        let (generation, place) = id.split_once(PART)?;
        if generation.parse::<u64>().ok()? != self.generation {
            return None;
        }
        self.shots.get(place.parse::<usize>().ok()?).cloned()
    }
}

/// The URL of the picture at `place` of `generation`.
pub(super) fn src(generation: u64, place: usize) -> String {
    format!("image://{SHOTS}/{generation}{PART}{place}")
}

/// The flattened `shot` cut to `page`'s kept part, as a grab comes in whole pixels of the window's scale and may hold a
/// little more at its right and bottom; a side short of the part stays as it is, with a warning, and any other shot
/// goes as it is.
pub(super) fn kept(shot: Shot, page: Option<&SnipPage>) -> Shot {
    let Some((w, h)) = part_of(page) else {
        return shot;
    };
    if shot.width() < w || shot.height() < h {
        tracing::warn!("a flattened picture is smaller than the kept part");
    }
    let (w, h) = (w.min(shot.width()), h.min(shot.height()));
    cut(&shot, w, h).unwrap_or(shot)
}

/// The size of `page`'s kept part while the editor shows.
fn part_of(page: Option<&SnipPage>) -> Option<(u32, u32)> {
    let SnipStage::Edit(editor) = &page?.stage else {
        return None;
    };
    let crop = editor.body.crop.rect();
    Some((u32::try_from(crop.w).ok()?, u32::try_from(crop.h).ok()?))
}

/// The top left `w` by `h` of `shot`, when that is less than all of it; else `None`.
fn cut(shot: &Shot, w: u32, h: u32) -> Option<Shot> {
    if (shot.width(), shot.height()) == (w, h) {
        return None;
    }
    let row = usize::try_from(w).ok()?.checked_mul(BGRX_BYTES)?;
    let stride = usize::try_from(shot.width())
        .ok()?
        .checked_mul(BGRX_BYTES)?;
    let from = shot.bgrx().expose();
    Shot::filled(w, h, |to| {
        let rows = to.chunks_exact_mut(row).zip(from.chunks_exact(stride));
        rows.for_each(|(t, f)| t.copy_from_slice(&f[..row]));
        true
    })
}

/// The frozen screens' pictures, then the editor's.
fn shots_of(page: &SnipPage) -> Vec<Shot> {
    let frozen = page.frozen.iter().map(|f| f.shot().clone());
    let edited = match &page.stage {
        SnipStage::Edit(editor) => Some(editor.picture.clone()),
        _ => None,
    };
    frozen.chain(edited).collect()
}
