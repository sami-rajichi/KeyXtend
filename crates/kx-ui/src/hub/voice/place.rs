//! Where the bar goes (design "The bar"): just above the keyboard and centred on it, else the bottom centre of its
//! screen, else the top centre, never over the keyboard while a spot is free; `place` tries the bottom or the top of
//! the screen first. Sizes come in Qt units and spots go out in physical pixels; the maths saturates.

use super::super::Hub;
use super::super::spots::size;
use crate::geom::{Size, centre, clamp, main_screen, screen_near};
use kx_module_api::ports::{BarPlace, Point, Rect};

/// A spot the bar may take.
#[derive(Copy, Clone)]
enum Spot {
    /// Above the keyboard, centred on it.
    Above,
    /// Below the keyboard, centred on it.
    Below,
    /// The bottom centre of the screen.
    Bottom,
    /// The top centre of the screen.
    Top,
}

/// The spots `place` tries, in order.
const fn order(place: BarPlace) -> [Spot; 3] {
    match place {
        BarPlace::Above => [Spot::Above, Spot::Bottom, Spot::Top],
        BarPlace::Bottom => [Spot::Bottom, Spot::Above, Spot::Top],
        BarPlace::Top => [Spot::Top, Spot::Below, Spot::Bottom],
    }
}

/// A box with edges wide enough that sums never overflow.
#[derive(Copy, Clone)]
struct Edges {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
}

impl Edges {
    /// `r`'s edges, moved `inset` inside it.
    fn of(r: Rect, inset: i32) -> Self {
        let (x, y, e) = (i64::from(r.x), i64::from(r.y), i64::from(inset));
        Self {
            left: x + e,
            top: y + e,
            right: x + i64::from(r.w) - e,
            bottom: y + i64::from(r.h) - e,
        }
    }

    /// True when the two boxes share a pixel.
    const fn meets(self, o: Self) -> bool {
        self.left < o.right && o.left < self.right && self.top < o.bottom && o.top < self.bottom
    }
}

impl Hub {
    /// Where a `w` by `h` bar goes for the shown view's place, by the keyboard `over` (physical pixels) or with none on
    /// the main screen, the theme's panel gaps from the keyboard and inside the work area; Above before any view.
    /// `None` when there is no screen.
    #[must_use]
    pub fn voice_at(&self, over: Option<Rect>, w: f64, h: f64) -> Option<Point> {
        let screens = self.ports.screens.work_areas();
        let screen = match over {
            Some(kb) => screen_near(centre(kb), &screens)?,
            None => main_screen(&screens)?,
        };
        let place = self.voice_place();
        let bar = size(screen, w, h);
        Some(at(bar, place, over, screen.rect, self.gaps(screen)))
    }
}

/// The top-left of a `bar` for `place` by keyboard `over` on work area `work`, `gap[0]` from the keyboard and `gap[1]`
/// inside the work area: the first spot that fits and stays off the keyboard, else the first spot, kept on the area.
fn at(bar: Size, place: BarPlace, over: Option<Rect>, work: Rect, gap: [i32; 2]) -> Point {
    let area = Edges::of(work, gap[1]);
    let kb = over.map(|r| Edges::of(r, 0));
    let spots = order(place).map(|s| spot(s, bar, kb, area, i64::from(gap[0])));
    let spots: Vec<Edges> = spots.into_iter().flatten().collect();
    let fits = |b: &&Edges| b.top >= area.top && b.bottom <= area.bottom;
    let free = |b: &&Edges| kb.is_none_or(|k| !b.meets(k));
    let chosen = spots.iter().find(|b| fits(b) && free(b));
    let first = chosen.or(spots.first()).copied().unwrap_or(area);
    Point {
        x: clamp(first.left),
        y: clamp(first.top.min(area.bottom - i64::from(bar.h)).max(area.top)),
    }
}

/// The box `bar` takes at `spot`, its left edge kept on `area`; `None` for a keyboard spot with no keyboard.
fn spot(spot: Spot, bar: Size, kb: Option<Edges>, area: Edges, space: i64) -> Option<Edges> {
    let (w, h) = (i64::from(bar.w), i64::from(bar.h));
    let mid = |l: i64, r: i64| l + (r - l - w) / 2;
    let (x, y) = match spot {
        Spot::Above => kb.map(|k| (mid(k.left, k.right), k.top - space - h))?,
        Spot::Below => kb.map(|k| (mid(k.left, k.right), k.bottom + space))?,
        Spot::Bottom => (mid(area.left, area.right), area.bottom - h),
        Spot::Top => (mid(area.left, area.right), area.top),
    };
    let x = x.min(area.right - w).max(area.left);
    Some(Edges {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    })
}
