//! `Ui`'s selection strip: the strip as JSON, its place from the size QML measured and the boxes of our windows, its
//! clicks sent on to the selection module, and the end of a ticked strip's fade.

use super::qobject;
use crate::bridges::qs;
use crate::geom::whole;
use crate::hub::{Hub, NULL, json};
use cxx_qt_lib::QString;
use kx_module_api::ports::Rect;

impl qobject::Ui {
    pub(super) fn sel(&self) -> QString {
        qs(&self.with(NULL.to_owned(), Hub::sel_json))
    }

    pub(super) fn sel_at(&self, round: i64, w: f64, h: f64, size: f64, clear: &QString) -> QString {
        let Ok(clear) = boxes(&clear.to_string()) else {
            tracing::warn!("QML sent boxes to keep the strip clear of that do not read");
            return qs(NULL);
        };
        let at = self.with(None, |hub| {
            hub.sel_at(round, whole(w), whole(h), size, &clear)
        });
        qs(&json(&at))
    }

    pub(super) fn sel_act(&self, round: i64, chip: &QString) {
        let chip = chip.to_string();
        self.with((), |h| h.sel_act(round, &chip));
    }

    pub(super) fn sel_faded(&self, round: i64) {
        self.with((), |h| h.sel_faded(round));
    }
}

/// The boxes of our windows QML sends in physical pixels, a JSON list; `null` is none.
fn boxes(json: &str) -> serde_json::Result<Vec<Rect>> {
    let read: Option<Vec<Rect>> = serde_json::from_str(json)?;
    Ok(read.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_boxes_read_as_a_rounded_list_or_null_and_refuse_the_rest() {
        let r = Rect {
            x: 10,
            y: 20,
            w: 300,
            h: 101,
        };
        let read = boxes(r#"[{"x": 10.2, "y": 19.6, "w": 300, "h": 100.5}]"#);
        assert_eq!(read.ok(), Some(vec![r]));
        assert_eq!(boxes("null").ok(), Some(Vec::new()), "none");
        assert_eq!(boxes("[]").ok(), Some(Vec::new()));
        for bad in [r#"{"x": 1}"#, r#"[{"x": 1}]"#, "oops", ""] {
            assert!(boxes(bad).is_err(), "{bad}");
        }
    }
}
