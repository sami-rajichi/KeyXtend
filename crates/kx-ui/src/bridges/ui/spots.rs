//! `Ui`'s pop-up spots: QML's keyboard box and sizes become the banner's, the lists' and the tray menu's spots, and the
//! clipboard panel's spot with its fit.

use super::qobject;
use crate::bridges::qs;
use crate::geom::whole;
use crate::hub::{NULL, json};
use cxx_qt_lib::QString;
use kx_module_api::ports::{Point, Rect};

impl qobject::Ui {
    pub(super) fn pop_at(
        &self,
        over: &QString,
        left: f64,
        centred: bool,
        w: f64,
        h: f64,
    ) -> QString {
        let Some((over, left)) = beside(over, left, centred) else {
            return qs(NULL);
        };
        let spot = self.with(None, |hub| hub.pop_at(over, left, w, h));
        qs(&json(&spot))
    }

    pub(super) fn pop_fit(
        &self,
        over: &QString,
        left: f64,
        centred: bool,
        w: f64,
        h: f64,
    ) -> QString {
        let Some((over, left)) = beside(over, left, centred) else {
            return qs(NULL);
        };
        let spot = self.with(None, |hub| hub.pop_fit(over, left, w, h));
        qs(&json(&spot))
    }

    pub(super) fn menu_at(&self, x: i32, y: i32, w: f64, h: f64) -> QString {
        let spot = self.with(None, |hub| hub.menu_at(Point { x, y }, w, h));
        qs(&json(&spot))
    }
}

/// The keyboard box QML sent and the pop-up's left edge, `None` for a centred one; `None` with a warning when the box
/// does not read.
fn beside(over: &QString, left: f64, centred: bool) -> Option<(Option<Rect>, Option<i32>)> {
    let Ok(over) = keyboard_box(&over.to_string()) else {
        tracing::warn!("QML sent a keyboard box that does not read");
        return None;
    };
    Some((over, (!centred).then(|| whole(left))))
}

/// The shown keyboard's box in physical pixels, `null` while it is hidden.
fn keyboard_box(json: &str) -> serde_json::Result<Option<Rect>> {
    serde_json::from_str(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_keyboard_box_reads_rounded_or_null_and_refuses_the_rest() {
        let r = Rect {
            x: 10,
            y: 20,
            w: 300,
            h: 101,
        };
        let read = keyboard_box(r#"{"x": 10.2, "y": 19.6, "w": 300, "h": 100.5}"#);
        assert_eq!(read.ok(), Some(Some(r)));
        assert_eq!(keyboard_box("null").ok(), Some(None), "hidden");
        assert!(keyboard_box("{\"x\": 1}").is_err());
        assert!(keyboard_box("oops").is_err());
    }
}
