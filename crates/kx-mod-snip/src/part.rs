//! The picking rules (design "Pictures"): what a click on the frozen screen gives for the way picked, the front
//! window or the screen under it, a video region kept to its first corner's screen, even and at least 2 by 2, and the
//! screen holding a part. The crop maths stay in `kx-module-api` (`Part::spanning`, `cut`).

use std::cmp::Reverse;

use kx_module_api::ports::{BarKey, Frozen, FrozenScreen, Part, Point, Rect};

/// What a click on the frozen screen gives for the way picked.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// A region's first corner.
    Corner(Point),
    /// The same spot clicked twice: the corner drops, and Region waits again.
    Unset,
    /// A part taken: a region, a window's frame or a screen.
    Part(Part),
}

/// The screen holding a part: where it lies in physical pixels, and its DPI, which scales the strip and the record bar.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Home {
    /// The whole screen, taskbar included.
    pub rect: Rect,
    /// Its DPI.
    pub dpi: u32,
}

/// H.264 takes frame sides in pairs of pixels, so a video region's sides are multiples of this.
pub(crate) const PAIR: u16 = 2;

/// A way of two corners.
const fn region(way: BarKey) -> bool {
    matches!(way, BarKey::Region | BarKey::VideoRegion)
}

/// What a click at `at` gives while `way` is picked, a region's first `corner` waiting or not; `None` when it gives
/// nothing, as a click on no window or between screens.
#[must_use]
pub fn read(way: BarKey, corner: Option<Point>, at: Point, frozen: &Frozen) -> Option<Pick> {
    if region(way) {
        return Some(match corner {
            None => Pick::Corner(at),
            Some(one) if one == at => Pick::Unset,
            Some(one) if way == BarKey::VideoRegion => Pick::Part(video(one, at, &frozen.screens)?),
            Some(one) => Pick::Part(Part::spanning(one, at)?),
        });
    }
    let part = match way {
        BarKey::Window => window(at, frozen)?,
        BarKey::Full | BarKey::VideoFull => Part::new(screen(at, &frozen.screens)?.rect())?,
        // The region ways read above; Settings and Close pick nothing.
        BarKey::Region | BarKey::VideoRegion | BarKey::Settings | BarKey::Close => return None,
    };
    Some(Pick::Part(part))
}

/// The front window whose frame holds `at`, cut down to what showed on the screens; `None` on no window.
#[must_use]
pub fn window(at: Point, frozen: &Frozen) -> Option<Part> {
    let frame = frozen
        .windows
        .iter()
        .map(|w| Edges::of(*w))
        .find(|w| w.holds(at))?;
    let shown = frozen
        .screens
        .iter()
        .filter_map(|s| frame.meet(Edges::of(s.rect())));
    Part::new(shown.reduce(Edges::join)?.rect()?)
}

/// The screen holding `at`; `None` between screens.
#[must_use]
pub fn screen(at: Point, screens: &[FrozenScreen]) -> Option<&FrozenScreen> {
    screens.iter().find(|s| Edges::of(s.rect()).holds(at))
}

/// The screen a region keeps to once its first `corner` is clicked: a video region's corner's, as the recorder takes
/// one screen; `None` for a still region, which crosses screens (edge 2).
#[must_use]
pub fn bound(way: BarKey, corner: Point, screens: &[FrozenScreen]) -> Option<Rect> {
    let video = way == BarKey::VideoRegion;
    video
        .then(|| screen(corner, screens))
        .flatten()
        .map(FrozenScreen::rect)
}

/// A video region from `one` to `at`, kept to the screen holding `one`, each side even and at least a pair, as the
/// recorder takes it; `None` when `one` lies on no screen.
fn video(one: Point, at: Point, screens: &[FrozenScreen]) -> Option<Part> {
    let on = Edges::of(screen(one, screens)?.rect());
    let span = Edges::of(Part::spanning(one, on.nearest(at)?)?.rect());
    let (left, right) = paired(span.left, span.right, on.right);
    let (top, bottom) = paired(span.top, span.bottom, on.bottom);
    let edges = Edges {
        left,
        top,
        right,
        bottom,
    };
    Part::new(edges.rect()?)
}

/// One side from `near` to `far`, its far edge out, made a whole number of pairs on a screen ending at `end`: an odd
/// side loses its far pixel, and a lone pixel grows away from the screen's edge.
fn paired(near: i64, far: i64, end: i64) -> (i64, i64) {
    let (len, pair) = (far - near, i64::from(PAIR));
    if len >= pair {
        (near, far - len % pair)
    } else if near + pair <= end {
        (near, near + pair)
    } else {
        (end - pair, end)
    }
}

/// The box a region's frame keeps within: a video region's screen, as the recorder takes one, or every screen together
/// for a picture's region, which may cross screens (edge 2); `None` for a window or a screen, which have no frame.
#[must_use]
pub fn within(way: BarKey, part: Part, screens: &[FrozenScreen]) -> Option<Rect> {
    let r = part.rect();
    match way {
        BarKey::VideoRegion => bound(way, Point { x: r.x, y: r.y }, screens),
        BarKey::Region => screens
            .iter()
            .map(|s| Edges::of(s.rect()))
            .reduce(Edges::join)?
            .rect(),
        BarKey::Window | BarKey::Full | BarKey::VideoFull | BarKey::Settings | BarKey::Close => {
            None
        }
    }
}

/// `rect` as a region's part once its frame moved: a video region's sides made even again on its screen, as `read`
/// takes them; `None` for an empty one or a video's on no screen.
#[must_use]
pub fn framed(way: BarKey, rect: Rect, screens: &[FrozenScreen]) -> Option<Part> {
    let part = Part::new(rect)?;
    if way != BarKey::VideoRegion {
        return Some(part);
    }
    let e = Edges::of(rect);
    let px = |v: i64| i32::try_from(v).ok();
    let far = Point {
        x: px(e.right - 1)?,
        y: px(e.bottom - 1)?,
    };
    video(
        Point {
            x: rect.x,
            y: rect.y,
        },
        far,
        screens,
    )
}

/// True when `at` lies on `part` or within `reach` pixels of its edge.
#[must_use]
pub fn near(part: Part, at: Point, reach: i32) -> bool {
    let (e, by) = (Edges::of(part.rect()), i64::from(reach));
    let grown = Edges {
        left: e.left - by,
        top: e.top - by,
        right: e.right + by,
        bottom: e.bottom + by,
    };
    grown.holds(at)
}

/// True when two rectangles share a pixel.
#[must_use]
pub fn meets(a: Rect, b: Rect) -> bool {
    Edges::of(a).meet(Edges::of(b)).is_some()
}

/// The screen holding the most of `part`, the first on a tie; `None` when it touches no screen.
#[must_use]
pub fn home(part: Part, screens: &[FrozenScreen]) -> Option<Home> {
    let part = Edges::of(part.rect());
    let shares = screens
        .iter()
        .filter_map(|s| Some((part.meet(Edges::of(s.rect()))?.area(), s)));
    let (_, best) = shares.min_by_key(|&(area, _)| Reverse(area))?;
    Some(Home {
        rect: best.rect(),
        dpi: best.dpi(),
    })
}

/// A rectangle's edges, wide enough that no sum of pixels overflows.
#[derive(Copy, Clone)]
struct Edges {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
}

impl Edges {
    fn of(r: Rect) -> Self {
        let (left, top) = (i64::from(r.x), i64::from(r.y));
        Self {
            left,
            top,
            right: left + i64::from(r.w),
            bottom: top + i64::from(r.h),
        }
    }

    /// Holds the pixel at `at`: its first row and column are in, its far edges are out.
    fn holds(self, at: Point) -> bool {
        let (x, y) = (i64::from(at.x), i64::from(at.y));
        (self.left..self.right).contains(&x) && (self.top..self.bottom).contains(&y)
    }

    /// The pixel it holds nearest `at`; `None` past the pixel range.
    fn nearest(self, at: Point) -> Option<Point> {
        let near =
            |v: i32, low: i64, high: i64| i32::try_from(i64::from(v).max(low).min(high - 1)).ok();
        Some(Point {
            x: near(at.x, self.left, self.right)?,
            y: near(at.y, self.top, self.bottom)?,
        })
    }

    /// The pixels both hold; `None` when they share none.
    fn meet(self, other: Self) -> Option<Self> {
        let both = Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        (both.left < both.right && both.top < both.bottom).then_some(both)
    }

    /// The least rectangle holding both.
    fn join(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    /// How many pixels it holds.
    fn area(self) -> i64 {
        (self.right - self.left).saturating_mul(self.bottom - self.top)
    }

    /// Back to a rectangle; `None` past the pixel range.
    fn rect(self) -> Option<Rect> {
        let px = |v: i64| i32::try_from(v).ok();
        Some(Rect {
            x: px(self.left)?,
            y: px(self.top)?,
            w: px(self.right - self.left)?,
            h: px(self.bottom - self.top)?,
        })
    }
}

#[cfg(test)]
mod tests;
