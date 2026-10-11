//! The keyboard window's own window calls: its cut to the plate, a tip lifted alone, and the no-focus guard.

use super::qobject;
use crate::geom::Area;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use std::pin::Pin;

impl qobject::Keyboard {
    pub(super) fn shape(&self, title: &QString, boxes: &QString, dpr: f64) -> bool {
        let Ok(boxes) = serde_json::from_str::<Vec<Area>>(&boxes.to_string()) else {
            tracing::warn!("QML sent boxes that do not read");
            return false;
        };
        let title = title.to_string();
        self.rust()
            .hub
            .as_deref()
            .is_some_and(|h| h.shape(&title, &boxes, dpr))
    }

    pub(super) fn lift_alone(&self, title: &QString) -> bool {
        let title = title.to_string();
        self.rust()
            .hub
            .as_deref()
            .is_some_and(|h| h.lift_alone(&title))
    }

    pub(super) fn guard(self: Pin<&mut Self>, top: &QString) -> bool {
        let top = top.to_string();
        let top = (!top.is_empty()).then_some(top.as_str());
        // No borrow lives across the try: the guard port moves Qt's windows, whose handlers may call this object.
        let (hub, attempt) = (self.rust().hub.clone(), self.rust().guard.next());
        let Some(hub) = hub else {
            return false;
        };
        let again = hub.guard(attempt, top);
        self.rust_mut().guard.settle(attempt, again);
        again
    }
}
