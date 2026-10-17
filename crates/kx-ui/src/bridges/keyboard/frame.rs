//! The keyboard window's press frame: drawn at once before the key acts (`press_frame = "now"`), the frame log's
//! marks around the key's action (`frame_log`), which hold no key, and the key sound after the key went out.

use super::qobject;
use crate::bridges::frames;
use crate::frames::Mark;
use cxx_qt_lib::QString;
use kx_module_api::ports::FaceAction;

impl qobject::Keyboard {
    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    pub(super) fn draw_now(&self, title: &QString) -> bool {
        frames::draw_now(title)
    }

    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    pub(super) fn press_start(&self) {
        frames::mark(Mark::Start);
    }

    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    pub(super) fn press_end(&self) {
        frames::mark(Mark::End);
    }

    /// Does a tap, a key's first press or a pad tap and answers what to redraw, as `run` does, with the frame log's
    /// mark once the key went out (`marked`), then the key sound.
    pub(super) fn sent(&self, action: FaceAction) -> QString {
        let mark = marked(&action);
        self.redraw(|h| h.sent(action, || mark.into_iter().for_each(frames::mark)))
    }
}

/// The frame log's mark once `action` went out: none for a pad tap, as Plate opens no press frame around it.
fn marked(action: &FaceAction) -> Option<Mark> {
    (!matches!(action, FaceAction::Pad { .. })).then_some(Mark::Sent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kx_module_api::ports::{Dir, KeyId};

    #[test]
    fn a_key_marks_sent_and_a_pad_tap_marks_nothing() {
        let id = || KeyId::new("a".to_owned());
        let tap = FaceAction::Tap {
            id: id(),
            back: false,
        };
        assert_eq!(marked(&tap), Some(Mark::Sent), "a tap");
        assert_eq!(
            marked(&FaceAction::Press { id: id() }),
            Some(Mark::Sent),
            "a first press"
        );
        for dir in Dir::ALL.map(Some).into_iter().chain([None]) {
            assert_eq!(marked(&FaceAction::Pad { dir }), None, "{dir:?}");
        }
    }
}
