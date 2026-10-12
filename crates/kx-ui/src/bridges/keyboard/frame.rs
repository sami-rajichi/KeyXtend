//! The keyboard window's press frame, drawn at once before the key acts (`press_frame = "now"`).

use super::qobject;
use crate::bridges::frames;
use cxx_qt_lib::QString;

impl qobject::Keyboard {
    #[allow(clippy::unused_self, reason = "cxx-qt invokables take the object")]
    pub(super) fn draw_now(&self, title: &QString) -> bool {
        frames::draw_now(title)
    }
}
