//! `Ui`'s wakes on the Qt thread: each tells QML what changed, a new theme's fonts load before QML reads the new look
//! and the last theme's go after, and only a look that changed has the windows drop what Qt cached for the last one.

use super::qobject;
use crate::bridges::device::release_after_frame;
use crate::hub::{Hub, Wake};
use cxx_qt::CxxQtType;
use kx_platform::ui::TrayEvent;
use std::pin::Pin;

/// Tells QML what changed, so no text asks Qt for a family it lacks: `fonts_changed` comes after the look's signal.
/// When the look QML reads is new, each shown window then drops what Qt cached for the last one once it drew it.
pub(super) fn woken(mut ui: Pin<&mut qobject::Ui>, wake: Wake) {
    let relook = matches!(wake, Wake::Settings | Wake::Look);
    let loaded = relook && ui.with(false, Hub::load_fonts);
    match wake {
        Wake::Settings => ui.as_mut().settings_changed(),
        Wake::Texts => ui.as_mut().texts_changed(),
        Wake::Look => ui.as_mut().look_changed(),
        Wake::Notices => ui.as_mut().notices_waiting(),
        Wake::Show => ui.as_mut().show_asked(),
        Wake::End => ui.as_mut().quit_asked(),
        Wake::Tray(TrayEvent::Click(at)) => ui.as_mut().tray_clicked(at.x, at.y),
        Wake::Tray(TrayEvent::Away) => ui.as_mut().tray_away(),
    }
    let dropped = relook && ui.with(false, Hub::drop_fonts);
    if loaded || dropped {
        ui.as_mut().fonts_changed();
    }
    if relook && ui.relooked() {
        release_after_frame();
    }
}

impl qobject::Ui {
    /// True when the look QML reads now differs from the one the windows drew before, which it keeps instead.
    fn relooked(mut self: Pin<&mut Self>) -> bool {
        let now = self.with(String::new(), Hub::look_json);
        if self.rust().drawn == now {
            return false;
        }
        self.as_mut().rust_mut().drawn = now;
        true
    }
}
