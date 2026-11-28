//! Where Snip's presses land and where its editor sits: a press on a frozen screen goes in physical desktop pixels by
//! that screen's own corner and scale, the editor's presses in its picture's pixels, and the editor's window stays on a
//! screen, no smaller than the theme's least (edge 19), and opens maximised again while its screen is there (edge 9).

use super::acts::round_of;
use crate::geom::{centre, clamp, fit_in, main_screen, screen_at, screen_holding, whole};
use crate::hub::Hub;
use kx_module_api::ports::{
    Point, Rect, Scale, Screen, SnipAct, SnipClick, SnipPage, SnipStage, physical,
};
use serde::Serialize;

/// Where the editor opens, in physical pixels.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EditorAt {
    /// Its window's box.
    pub place: Rect,
    /// The box it comes back to when it opens maximised; `None` when it opens unmaximised.
    pub back: Option<Rect>,
}

impl Hub {
    /// A press at (`x`, `y`) logical pixels on frozen screen `screen` of view `round`, sent in physical desktop pixels;
    /// no such screen, or a point that does not read, sends nothing. Call it on the Qt thread.
    pub fn snip_spot(&self, screen: i32, x: f64, y: f64, round: i64) {
        let at = self.snip_page().and_then(|p| spot(&p, screen, x, y));
        if let (Some(at), Ok(round)) = (at, round_of(round)) {
            let click = SnipClick::Spot(at);
            self.snip_send(&SnipAct { round, click });
        } else {
            tracing::warn!("QML sent a press on no frozen screen, or one that does not read");
        }
    }

    /// A press at (`x`, `y`) window pixels on the editor's picture, shown at `scale` window pixels a picture pixel.
    pub fn snip_press(&self, x: f64, y: f64, scale: f64, round: i64) {
        let press = shown(scale).zip(picture(x, y, scale));
        if let (Some((scale, at)), Ok(round)) = (press, round_of(round)) {
            let click = SnipClick::Press { at, scale };
            self.snip_send(&SnipAct { round, click });
        } else {
            tracing::warn!("QML sent an editor press that does not read");
        }
    }

    /// A pen's path since its first click, in window pixels on the picture shown at `scale`; an empty one sends
    /// nothing.
    pub fn snip_stroke(&self, points: &[[f64; 2]], scale: f64, round: i64) {
        let path = shown(scale).and_then(|_| {
            let path: Option<Vec<Point>> =
                points.iter().map(|[x, y]| picture(*x, *y, scale)).collect();
            path.filter(|p| !p.is_empty())
        });
        if let (Some(path), Ok(round)) = (path, round_of(round)) {
            let click = SnipClick::Stroke(path);
            self.snip_send(&SnipAct { round, click });
        } else {
            tracing::warn!("QML sent a pen path that does not read");
        }
    }

    /// Where the editor opens, in physical pixels: where it was last placed, else at the theme's size centred on the
    /// pointer's screen, kept on a screen and no smaller than its least; or maximised, as `editor_maximised` says.
    /// `None` with no screen. Qt thread only.
    #[must_use]
    pub fn editor_at(&self) -> Option<EditorAt> {
        let screens = self.ports.screens.work_areas();
        let (kept, full) = self.snip_page().map_or((None, None), |p| match &p.stage {
            SnipStage::Edit(editor) => (editor.place, editor.full),
            _ => (None, None),
        });
        if let Some(full) = full {
            return self.editor_maximised(kept, full, &screens);
        }
        let wanted = kept.or_else(|| self.editor_default(&screens))?;
        let place = self.editor_on(wanted, &screens)?;
        Some(EditorAt { place, back: None })
    }

    /// The editor moved or resized to `rect`, physical pixels, kept on its screen and no smaller than its least; a step
    /// of a move or a resize tells the module nothing.
    #[must_use]
    pub fn editor_fit(&self, rect: Rect) -> Option<Rect> {
        self.editor_on(rect, &self.ports.screens.work_areas())
    }

    /// The editor ended a move, a resize or a restore at `rect`: kept as `editor_fit` keeps it, and the module keeps
    /// that place, unmaximised, for next time.
    pub fn editor_placed(&self, rect: Rect) -> Option<Rect> {
        let screens = self.ports.screens.work_areas();
        let rect = self.editor_on(rect, &screens)?;
        self.snip_now(SnipClick::Placed {
            rect,
            full: None,
            screens,
        });
        Some(rect)
    }

    /// The editor maximised from `rect`: its screen's work area, which the module keeps with `rect` under it for next
    /// time; `None` with no screen.
    pub fn editor_full(&self, rect: Rect) -> Option<Rect> {
        let screens = self.ports.screens.work_areas();
        let rect = self.editor_on(rect, &screens)?;
        let work = screen_at(centre(rect), &screens)?.rect;
        self.snip_now(SnipClick::Placed {
            rect,
            full: Some(work),
            screens,
        });
        Some(work)
    }

    /// A maximised editor: over the work area of the screen it filled, as it is now, with its box under it. With that
    /// screen gone, unmaximised at the theme's size at the main screen's own scale, centred on it (edge 9).
    fn editor_maximised(
        &self,
        kept: Option<Rect>,
        full: Rect,
        screens: &[Screen],
    ) -> Option<EditorAt> {
        let Some(screen) = screen_holding(centre(full), screens) else {
            let place = self.editor_on(self.editor_centred(main_screen(screens)?), screens)?;
            return Some(EditorAt { place, back: None });
        };
        let back = kept.unwrap_or_else(|| self.editor_centred(screen));
        Some(EditorAt {
            place: screen.rect,
            back: Some(self.editor_on(back, screens)?),
        })
    }

    /// The theme's editor size at the pointer's screen's scale, centred on it.
    fn editor_default(&self, screens: &[Screen]) -> Option<Rect> {
        let screen = match self.cursor() {
            Some(at) => screen_at(at, screens),
            None => main_screen(screens),
        }?;
        Some(self.editor_centred(screen))
    }

    /// The theme's editor size at `screen`'s scale, centred on its work area.
    fn editor_centred(&self, screen: &Screen) -> Rect {
        let [w, h] = self
            .themes
            .shape
            .snip
            .editor_px
            .map(|v| physical(f64::from(v), screen.dpi));
        let mid = |at: i32, len: i32, side: i32| {
            clamp(i64::from(at) + (i64::from(len) - i64::from(side)) / 2)
        };
        let r = screen.rect;
        Rect {
            x: mid(r.x, r.w, w),
            y: mid(r.y, r.h, h),
            w,
            h,
        }
    }

    /// `rect` on the screen under its middle: no smaller than the theme's least at that screen's scale, no larger than
    /// its work area, and wholly inside it.
    fn editor_on(&self, rect: Rect, screens: &[Screen]) -> Option<Rect> {
        let screen = screen_at(centre(rect), screens)?;
        let least = self
            .themes
            .shape
            .snip
            .editor_min_px
            .map(|v| physical(f64::from(v), screen.dpi));
        Some(kept_in(rect, screen.rect, least))
    }
}

/// `rect` no smaller than `least` and no larger than `work`, moved wholly inside it.
fn kept_in(rect: Rect, work: Rect, [lw, lh]: [i32; 2]) -> Rect {
    let (w, h) = (rect.w.max(lw).min(work.w), rect.h.max(lh).min(work.h));
    let along = |at: i32, len: i32, from: i32, room: i32| {
        let last = i64::from(from) + i64::from(room) - i64::from(len);
        fit_in(i64::from(at), i64::from(from), last)
    };
    Rect {
        x: along(rect.x, w, work.x, work.w),
        y: along(rect.y, h, work.y, work.h),
        w,
        h,
    }
}

/// A press at (`x`, `y`) logical pixels on frozen screen `screen` of `page`, in physical desktop pixels: the screen's
/// corner, which Qt keeps unscaled, plus the press at the screen's own scale.
fn spot(page: &SnipPage, screen: i32, x: f64, y: f64) -> Option<Point> {
    let frozen = page.frozen.get(usize::try_from(screen).ok()?)?;
    let (corner, dpi) = (frozen.rect(), frozen.dpi());
    let at = |v: f64, from: i32| {
        let px = i64::from(from) + i64::from(physical(v, dpi));
        v.is_finite().then(|| clamp(px))
    };
    Some(Point {
        x: at(x, corner.x)?,
        y: at(y, corner.y)?,
    })
}

/// How large the picture shows, as the module counts it: window pixels for every `Scale::PER` of its own, rounded up so
/// never 0; `None` for a scale that does not read.
fn shown(scale: f64) -> Option<Scale> {
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let per = whole((scale * f64::from(Scale::PER)).ceil());
    Scale::new(u32::try_from(per).ok()?.max(1))
}

/// (`x`, `y`) window pixels on a picture shown at `scale`, in the picture's pixels.
fn picture(x: f64, y: f64, scale: f64) -> Option<Point> {
    (x.is_finite() && y.is_finite()).then(|| Point {
        x: whole(x / scale),
        y: whole(y / scale),
    })
}
