//! The keyboard window's scroll pad: its clicks, which sound like a key but leave no frame log mark, and the pointer on
//! it go to the face, which publishes them on the bus, each answering `{board, state}`.

use super::qobject;
use crate::bridges::qs;
use crate::hub::Redraw;
use cxx_qt_lib::QString;
use kx_module_api::ports::{Dir, FaceAction};

impl qobject::Keyboard {
    pub(super) fn pad(&self, dir: &QString) -> QString {
        let Ok(dir) = way(&dir.to_string()) else {
            tracing::warn!("QML named no scroll pad arrow");
            return qs(&Redraw::default().to_json());
        };
        self.sent(FaceAction::Pad { dir })
    }

    pub(super) fn pad_hover(&self, on: bool) -> QString {
        self.run(FaceAction::PadHover { on })
    }
}

/// The arrow `name` names, or none for Stop when it is empty.
fn way(name: &str) -> serde_json::Result<Option<Dir>> {
    if name.is_empty() {
        return Ok(None);
    }
    serde_json::from_value(serde_json::Value::String(name.to_owned())).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_arrow_reads_by_its_name_and_empty_is_stop() {
        for dir in Dir::ALL {
            let name = serde_json::to_value(dir).unwrap();
            assert_eq!(way(name.as_str().unwrap()).ok(), Some(Some(dir)));
        }
        assert_eq!(way("").ok(), Some(None), "Stop");
        assert!(way("Up").is_err() && way("north").is_err() && way(" ").is_err());
    }
}
