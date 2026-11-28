//! The crop maths, moved from the spike's `capture.rs` (G24): the part between two clicked corners, and a part's pixels
//! cut from the frozen screens in physical pixels, so a part across screens of any scaling keeps exact pixels.

use super::shot::{BGRX_BYTES, FrozenScreen, Part, Shot};
use crate::ports::{Point, Rect};

impl Part {
    /// The part between corners `one` and `two`, clicked in either order, both corner pixels inside; `None` when it
    /// passes the pixel range.
    #[must_use]
    pub fn spanning(one: Point, two: Point) -> Option<Self> {
        let (x, y) = (one.x.min(two.x), one.y.min(two.y));
        let side = |near: i32, far: i32| far.checked_sub(near)?.checked_add(1);
        let (w, h) = (side(x, one.x.max(two.x))?, side(y, one.y.max(two.y))?);
        Self::new(Rect { x, y, w, h })
    }

    /// This part's pixels cut from `screens`, a pixel on no screen black. `None` when it touches no screen or holds
    /// more bytes than `screens` together, so a cut never outgrows its freeze or the freeze's cap.
    #[must_use]
    pub fn cut(self, screens: &[FrozenScreen]) -> Option<Shot> {
        let rect = self.rect();
        let (w, h) = (u32::try_from(rect.w).ok()?, u32::try_from(rect.h).ok()?);
        let len = usize::try_from(w)
            .ok()?
            .checked_mul(usize::try_from(h).ok()?)?
            .checked_mul(BGRX_BYTES)?;
        let frozen = screens.iter().map(|s| s.shot().bgrx().expose().len());
        if len > frozen.fold(0, usize::saturating_add) {
            return None;
        }
        let mut bgrx = vec![0; len];
        let mut touched = false;
        for screen in screens {
            touched |= copy(screen, rect, &mut bgrx).is_some();
        }
        touched.then_some(())?;
        Shot::new(w, h, bgrx)
    }
}

/// Copies the rows of `screen` inside `part` into `out`, which holds `part`'s pixels; `None` when they do not overlap.
fn copy(screen: &FrozenScreen, part: Rect, out: &mut [u8]) -> Option<()> {
    let s = screen.rect();
    let far = |at: i32, len: i32| i64::from(at) + i64::from(len);
    let (left, top) = (i64::from(part.x.max(s.x)), i64::from(part.y.max(s.y)));
    let right = far(part.x, part.w).min(far(s.x, s.w));
    let bottom = far(part.y, part.h).min(far(s.y, s.h));
    if right <= left || bottom <= top {
        return None;
    }
    let bytes = |px: i64| usize::try_from(px).ok()?.checked_mul(BGRX_BYTES);
    let row = bytes(right - left)?;
    let pixels = screen.shot().bgrx().expose();
    for y in top..bottom {
        let from = bytes((y - i64::from(s.y)) * i64::from(s.w) + left - i64::from(s.x))?;
        let to = bytes((y - i64::from(part.y)) * i64::from(part.w) + left - i64::from(part.x))?;
        let line = pixels.get(from..from.checked_add(row)?)?;
        out.get_mut(to..to.checked_add(row)?)?.copy_from_slice(line);
    }
    Some(())
}
