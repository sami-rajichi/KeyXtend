//! Where the pop-ups go: the banner and the language list beside the keyboard, and the tray menu by its icon.
//! Sizes come in Qt units and spots go out in physical pixels, at the scale of the screen they land on.

use super::Hub;
use crate::geom::{Size, beside, centre, clamp, home, main_screen, menu, screen_at, screen_near};
use kx_module_api::ports::{Point, Rect, Screen, physical};

impl Hub {
    /// Where a `w` by `h` pop-up goes beside the keyboard `over` (physical pixels), its left edge at `left` or centred
    /// when `None`; with no keyboard, at the bottom centre of the main screen. `None` when there is no screen.
    #[must_use]
    pub fn pop_at(&self, over: Option<Rect>, left: Option<i32>, w: f64, h: f64) -> Option<Point> {
        let screens = self.ports.screens.work_areas();
        let Some(over) = over else {
            let main = main_screen(&screens)?;
            let [_, edge] = self.gaps(main);
            return Some(home(size(main, w, h), main.rect, edge));
        };
        let mid = centre(over);
        let screen = screen_near(mid, &screens)?;
        let size = size(screen, w, h);
        let left = left.unwrap_or_else(|| clamp(i64::from(mid.x) - i64::from(size.w) / 2));
        Some(beside(size, left, over, screen.rect, self.gaps(screen)))
    }

    /// Where a `w` by `h` menu goes for a tray click at `at` (physical pixels): above and right of it, inside the work
    /// area that holds it, else the main screen's, where Windows keeps tray icons. `None` when there is no screen.
    #[must_use]
    pub fn menu_at(&self, at: Point, w: f64, h: f64) -> Option<Point> {
        let screens = self.ports.screens.work_areas();
        let screen = screen_at(at, &screens)?;
        let [_, edge] = self.gaps(screen);
        Some(menu(size(screen, w, h), at, screen.rect, edge))
    }

    /// The theme's gaps to the keyboard and to the screen edge, in `screen`'s pixels.
    fn gaps(&self, screen: &Screen) -> [i32; 2] {
        let gap = self.themes.shape.panel.gap_px;
        gap.map(|g| physical(f64::from(g), screen.dpi))
    }
}

/// `w` by `h` Qt units in `screen`'s pixels.
fn size(screen: &Screen, w: f64, h: f64) -> Size {
    Size {
        w: physical(w, screen.dpi),
        h: physical(h, screen.dpi),
    }
}
