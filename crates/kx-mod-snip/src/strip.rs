//! Where the strip and the record bar go (design "The strip", "Videos"): the strip under the part, above it when there is
//! no room, else at the bottom centre; the record bar above the part, else at the top centre. Room is judged on the
//! screen holding the part, so both stay on it; the ui centres them there.

use kx_module_api::ports::{Part, Rect, SnipSpot, physical};

/// A bar's height and its gap from the part, in physical pixels; only `scaled` makes one, so neither is negative.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Size {
    h: i32,
    gap: i32,
}

impl Size {
    /// The theme's height `h` and `gap`, in logical pixels, at `dpi`.
    #[must_use]
    pub fn scaled(h: u16, gap: u16, dpi: u32) -> Self {
        let px = |v: u16| physical(f64::from(v), dpi);
        Self {
            h: px(h),
            gap: px(gap),
        }
    }

    /// The height.
    #[must_use]
    pub const fn h(self) -> i32 {
        self.h
    }

    /// The gap between the part and the bar.
    #[must_use]
    pub const fn gap(self) -> i32 {
        self.gap
    }
}

/// Where the strip goes on `screen`, the one holding `part`: under it, above it when there is no room under, else at
/// the bottom centre, as for a full screen.
#[must_use]
pub fn place(part: Part, screen: Rect, size: Size) -> SnipSpot {
    if fits(under(part, size), screen, size) {
        SnipSpot::Under
    } else if fits(above(part, size), screen, size) {
        SnipSpot::Above
    } else {
        SnipSpot::Bottom
    }
}

/// Where the record bar goes on `screen`, the one holding `part`: above it, else at the top centre, as for a full screen.
#[must_use]
pub fn bar(part: Part, screen: Rect, size: Size) -> SnipSpot {
    if fits(above(part, size), screen, size) {
        SnipSpot::Above
    } else {
        SnipSpot::Top
    }
}

/// The top row of a bar `gap` under `part`.
fn under(part: Part, size: Size) -> i64 {
    let r = part.rect();
    i64::from(r.y) + i64::from(r.h) + i64::from(size.gap)
}

/// The top row of a bar `gap` above `part`.
fn above(part: Part, size: Size) -> i64 {
    i64::from(part.rect().y) - i64::from(size.gap) - i64::from(size.h)
}

/// A bar whose top row is `top` lies on `screen` from its top row to its bottom one.
fn fits(top: i64, screen: Rect, size: Size) -> bool {
    let (first, end) = (
        i64::from(screen.y),
        i64::from(screen.y) + i64::from(screen.h),
    );
    top >= first && top + i64::from(size.h) <= end
}

#[cfg(test)]
mod tests;
