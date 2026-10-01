//! Where the keyboard window goes, in physical pixels: the user's last place while it is on a screen, else the
//! bottom centre of the primary screen, always inside a work area. Each screen's DPI sets the window's size there.

use crate::settings::KeyboardSettings;
use kx_module_api::ports::{Point, Rect, Screen, physical};
use kx_module_api::{SettingsHandle, WriteError};

/// The settings key of the saved left edge.
const X_KEY: &str = "x";
/// The settings key of the saved top edge.
const Y_KEY: &str = "y";
/// The settings key that says the place was saved.
const PLACED_KEY: &str = "placed";

/// `v` as a pixel, stopping at the ends of the range.
fn px(v: i64) -> i32 {
    i32::try_from(v).unwrap_or(if v < 0 { i32::MIN } else { i32::MAX })
}

/// `pos` moved so a `len` span stays inside `start..start + span`, or `start` when it cannot fit.
fn axis(pos: i32, len: i32, start: i32, span: i32) -> i32 {
    let far = i64::from(start) + i64::from(span) - i64::from(len);
    px(i64::from(pos).min(far).max(i64::from(start)))
}

/// The length two spans share, `a..a + a_len` and `b..b + b_len`; 0 when they do not meet.
fn shared(a: i32, a_len: i32, b: i32, b_len: i32) -> i64 {
    let lo = i64::from(a).max(i64::from(b));
    let hi = (i64::from(a) + i64::from(a_len)).min(i64::from(b) + i64::from(b_len));
    (hi - lo).max(0)
}

/// A window `size` logical pixels big, in physical pixels on a screen at `dpi`.
fn sized((w, h): (f64, f64), dpi: u32) -> (i32, i32) {
    (physical(w, dpi), physical(h, dpi))
}

/// The top-left that keeps a `size` box at `at` inside `work`; a box bigger than `work` starts at its top-left.
#[must_use]
pub fn inside(at: Point, size: (i32, i32), work: Rect) -> Point {
    Point {
        x: axis(at.x, size.0, work.x, work.w),
        y: axis(at.y, size.1, work.y, work.h),
    }
}

/// The top-left of a `size` box at the bottom centre of `work`, inside it.
#[must_use]
pub fn bottom_centre(work: Rect, size: (i32, i32)) -> Point {
    let x = i64::from(work.x) + (i64::from(work.w) - i64::from(size.0)) / 2;
    let y = i64::from(work.y) + i64::from(work.h) - i64::from(size.1);
    inside(Point { x: px(x), y: px(y) }, size, work)
}

/// The bottom centre of the primary screen, else the first, for a window `size` logical pixels big;
/// `None` with no screen.
#[must_use]
pub fn home(size: (f64, f64), screens: &[Screen]) -> Option<Point> {
    let main = screens.iter().find(|s| s.primary).or(screens.first())?;
    Some(bottom_centre(main.rect, sized(size, main.dpi)))
}

/// The screen that holds the largest share of a window `size` logical pixels big at `at`, sized at each screen's
/// DPI; `None` when it is off every screen.
#[must_use]
pub fn screen_of(at: Point, size: (f64, f64), screens: &[Screen]) -> Option<&Screen> {
    let share = |s: &Screen| {
        let (w, h) = sized(size, s.dpi);
        let covered = shared(at.x, w, s.rect.x, s.rect.w) * shared(at.y, h, s.rect.y, s.rect.h);
        let whole = i64::from(w.max(1)) * i64::from(h.max(1));
        (i128::from(covered), i128::from(whole))
    };
    let on = screens.iter().map(|s| (share(s), s));
    let on = on.filter(|((covered, _), _)| *covered > 0);
    let best = on.max_by(|((ac, aw), _), ((bc, bw), _)| (ac * bw).cmp(&(bc * aw)));
    best.map(|(_, s)| s)
}

/// Where a window `size` logical pixels big with its top-left at `at` goes: inside the screen that holds the
/// largest share of it, at that screen's DPI, else home when it is off every screen.
#[must_use]
pub fn fit(at: Point, size: (f64, f64), screens: &[Screen]) -> Option<Point> {
    match screen_of(at, size, screens) {
        Some(s) => Some(inside(at, sized(size, s.dpi), s.rect)),
        None => home(size, screens),
    }
}

/// Where the keyboard starts: its `saved` place fitted to the screens, else home.
#[must_use]
pub fn start(saved: Option<Point>, size: (f64, f64), screens: &[Screen]) -> Option<Point> {
    match saved {
        Some(at) => fit(at, size, screens),
        None => home(size, screens),
    }
}

/// The keyboard's place, saved through the module's own settings handle when the user moves it.
#[derive(Debug)]
pub struct Placer {
    saved: Option<Point>,
    at: Option<Point>,
    store: SettingsHandle,
}

impl Placer {
    /// The place saved in `settings`, which `store` saves again after each move.
    #[must_use]
    pub fn new(settings: &KeyboardSettings, store: SettingsHandle) -> Self {
        Self {
            saved: settings.place(),
            at: None,
            store,
        }
    }

    /// Where the keyboard is now, once started or moved.
    #[must_use]
    pub fn at(&self) -> Option<Point> {
        self.at
    }

    /// The place the settings hold, if the user ever put the keyboard somewhere.
    #[must_use]
    pub fn saved(&self) -> Option<Point> {
        self.saved
    }

    /// Puts the keyboard where it starts, saving nothing, so the saved place comes back with its screen.
    pub fn start(&mut self, size: (f64, f64), screens: &[Screen]) -> Option<Point> {
        self.at = start(self.saved, size, screens);
        self.at
    }

    /// The user put the keyboard at `at`: it stays on a screen, and a new place is saved.
    /// It holds even when saving fails; with no screen it gives `None` and saves nothing.
    ///
    /// # Errors
    /// `Unsaved` when the file was not written: the store still keeps the place, now the saved one, and saves it
    /// later. Any other `WriteError` keeps none of it.
    pub fn moved(
        &mut self,
        at: Point,
        size: (f64, f64),
        screens: &[Screen],
    ) -> Result<Option<Point>, WriteError> {
        let Some(p) = fit(at, size, screens) else {
            return Ok(None);
        };
        self.at = Some(p);
        if self.saved != Some(p) {
            let wrote = self.save(p);
            if wrote.as_ref().err().is_none_or(WriteError::holds) {
                self.saved = Some(p);
            }
            wrote?;
        }
        Ok(Some(p))
    }

    /// Writes `p` and marks it placed in one write, so a place is never saved in part.
    fn save(&self, p: Point) -> Result<(), WriteError> {
        let values = [
            (X_KEY, toml::Value::Integer(i64::from(p.x))),
            (Y_KEY, toml::Value::Integer(i64::from(p.y))),
            (PLACED_KEY, toml::Value::Boolean(true)),
        ];
        self.store
            .set_all(values.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }
}

#[cfg(test)]
mod tests;
