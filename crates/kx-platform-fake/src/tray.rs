//! A fake tray: the test reads what each shown icon holds and fires its clicks and "elsewhere" events.

use crate::lock;
use kx_module_api::ports::Point;
use kx_platform::ui::{OnTray, Tray, TrayEvent, TrayIcon, TrayImage, UiError};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};

/// One shown icon: its picture, its tooltip and its listener.
struct Live {
    image: TrayImage,
    tip: String,
    on: OnTray,
}

/// What the tray and its icons share: the icons that show, by id, and the error the test set.
#[derive(Default)]
struct Shared {
    icons: Mutex<BTreeMap<u64, Live>>,
    error: Mutex<Option<UiError>>,
}

impl Shared {
    /// The error the test set, if any.
    fn refused(&self) -> Result<(), UiError> {
        lock(&self.error).map_or(Ok(()), Err)
    }
}

/// A tray the test drives; each event calls every icon that still shows.
pub struct FakeTray {
    px: u32,
    next: AtomicU64,
    shared: Arc<Shared>,
}

impl FakeTray {
    /// A tray whose pictures are `px` pixels square, with no icon.
    #[must_use]
    pub fn new(px: u32) -> Self {
        Self {
            px,
            next: AtomicU64::default(),
            shared: Arc::default(),
        }
    }

    /// Refuses later `show` and `set` calls with `error`, or takes them again with `None`.
    pub fn set_error(&self, error: Option<UiError>) {
        *lock(&self.shared.error) = error;
    }

    /// The picture and tooltip of each icon that shows, oldest first.
    #[must_use]
    pub fn shown(&self) -> Vec<(TrayImage, String)> {
        let icons = lock(&self.shared.icons);
        icons
            .values()
            .map(|l| (l.image.clone(), l.tip.clone()))
            .collect()
    }

    /// A click on the icons at `at`; the tray is locked meanwhile, so a listener must not touch its icon or the tray.
    pub fn click(&self, at: Point) {
        self.tell(TrayEvent::Click(at));
    }

    /// Another window came to the front.
    pub fn away(&self) {
        self.tell(TrayEvent::Away);
    }

    fn tell(&self, event: TrayEvent) {
        for live in lock(&self.shared.icons).values() {
            (live.on)(event);
        }
    }
}

impl fmt::Debug for FakeTray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeTray")
            .field("px", &self.px)
            .field("icons", &lock(&self.shared.icons).len())
            .finish_non_exhaustive()
    }
}

impl Tray for FakeTray {
    fn icon_px(&self) -> u32 {
        self.px
    }

    fn show(&self, image: &TrayImage, tip: &str, on: OnTray) -> Result<Box<dyn TrayIcon>, UiError> {
        self.shared.refused()?;
        let id = self.next.fetch_add(1, Ordering::SeqCst);
        let live = Live {
            image: image.clone(),
            tip: tip.to_owned(),
            on,
        };
        lock(&self.shared.icons).insert(id, live);
        Ok(Box::new(FakeIcon {
            id,
            shared: Arc::downgrade(&self.shared),
        }))
    }
}

/// A shown fake icon; dropping it removes it.
struct FakeIcon {
    id: u64,
    shared: Weak<Shared>,
}

impl TrayIcon for FakeIcon {
    fn set(&self, image: &TrayImage, tip: &str) -> Result<(), UiError> {
        let Some(shared) = self.shared.upgrade() else {
            return Ok(());
        };
        if let Some(live) = lock(&shared.icons).get_mut(&self.id) {
            live.image = image.clone();
            tip.clone_into(&mut live.tip);
        }
        // Kept even when refused, as the real taskbar shows it when it restarts.
        shared.refused()
    }
}

impl Drop for FakeIcon {
    fn drop(&mut self) {
        if let Some(shared) = self.shared.upgrade() {
            lock(&shared.icons).remove(&self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(fill: u8) -> TrayImage {
        TrayImage::new(1, 1, vec![fill; 4]).unwrap()
    }

    fn heard() -> (Arc<Mutex<Vec<TrayEvent>>>, OnTray) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let keep = Arc::clone(&seen);
        (seen, Box::new(move |e| keep.lock().unwrap().push(e)))
    }

    #[test]
    fn an_icon_shows_until_it_drops_and_hears_every_event_meanwhile() {
        let tray = FakeTray::new(20);
        assert_eq!(tray.icon_px(), 20);
        let (seen, on) = heard();
        let icon = tray.show(&image(1), "KeyXtend", on).unwrap();
        assert_eq!(tray.shown(), [(image(1), "KeyXtend".to_owned())]);
        tray.click(Point { x: 5, y: 6 });
        tray.away();
        let want = [TrayEvent::Click(Point { x: 5, y: 6 }), TrayEvent::Away];
        assert_eq!(*seen.lock().unwrap(), want);
        drop(icon);
        assert!(tray.shown().is_empty());
        tray.away();
        assert_eq!(seen.lock().unwrap().len(), 2, "nothing after the drop");
    }

    #[test]
    fn set_changes_the_picture_and_the_tip_and_keeps_a_refused_one() {
        let tray = FakeTray::new(16);
        let icon = tray.show(&image(1), "a", Box::new(|_| {})).unwrap();
        icon.set(&image(2), "b").unwrap();
        assert_eq!(tray.shown(), [(image(2), "b".to_owned())]);
        tray.set_error(Some(UiError::Refused(3)));
        assert_eq!(icon.set(&image(3), "c"), Err(UiError::Refused(3)));
        assert_eq!(
            tray.shown(),
            [(image(3), "c".to_owned())],
            "kept for a restart"
        );
    }

    #[test]
    fn a_refused_show_makes_no_icon() {
        let tray = FakeTray::new(16);
        tray.set_error(Some(UiError::Refused(5)));
        let shown = tray.show(&image(1), "a", Box::new(|_| {}));
        assert!(matches!(shown, Err(UiError::Refused(5))));
        assert!(tray.shown().is_empty());
    }

    #[test]
    fn an_icon_may_outlive_its_tray() {
        let tray = FakeTray::new(16);
        let icon = tray.show(&image(1), "a", Box::new(|_| {})).unwrap();
        drop(tray);
        assert_eq!(icon.set(&image(2), "b"), Ok(()));
        drop(icon);
    }
}
