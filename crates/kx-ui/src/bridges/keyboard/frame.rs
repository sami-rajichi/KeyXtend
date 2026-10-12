//! The keyboard window's press frame: drawn at once before the key acts (`press_frame = "now"`), and the frame log's
//! marks around the key's action (`frame_log`), which hold no key.

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

    /// Does key `action` and answers what to redraw, as `run` does, with the frame log's mark once the key went out.
    pub(super) fn sent(&self, action: FaceAction) -> QString {
        self.redraw(|h| {
            let redraw = h.act(action);
            frames::mark(Mark::Sent);
            redraw
        })
    }
}
