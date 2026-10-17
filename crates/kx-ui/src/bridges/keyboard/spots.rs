//! The keyboard window's points: the pointer, and the bubble's spot in a bottom corner, in physical pixels.

use super::qobject;
use crate::bridges::qs;
use crate::geom::whole;
use crate::hub::{NULL, json};
use cxx_qt_lib::QString;
use kx_module_api::ports::{Corner, Point};

impl qobject::Keyboard {
    pub(super) fn cursor(&self) -> QString {
        self.with(NULL, |h| json(&h.cursor()))
    }

    pub(super) fn bubble_at(&self, x: f64, y: f64, corner: &QString) -> QString {
        let at = Point {
            x: whole(x),
            y: whole(y),
        };
        let corner = serde_json::Value::String(corner.to_string());
        let Ok(corner) = serde_json::from_value::<Corner>(corner) else {
            tracing::warn!("QML named no bottom corner");
            return qs(NULL);
        };
        self.with(NULL, |h| json(&h.bubble_at(at, corner)))
    }
}
