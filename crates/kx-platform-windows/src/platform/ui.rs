//! The UI ports `WindowsPlatform` hands the UI: the guard, screens, look, backdrops, one running copy, tray, memory,
//! reveal, hidden-run probe, speaker and the Esc watch.

use std::sync::Arc;

use kx_platform::ui::{
    Backdrop, EscWatch, Hidden, Instance, Memory, Reveal, Screens, Speaker, SystemLook, Tray,
    WindowGuard,
};

use super::WindowsPlatform;
use crate::{
    WinBackdrop, WinEscWatch, WinGuard, WinHidden, WinInstance, WinLook, WinMemory, WinReveal,
    WinScreens, WinSpeaker, WinTray,
};

impl WindowsPlatform {
    /// The window guard, for the UI to call on the thread that owns our windows.
    #[must_use]
    pub fn guard(&self) -> Arc<dyn WindowGuard> {
        Arc::new(WinGuard)
    }

    /// The screens and the pointer.
    #[must_use]
    pub fn screens(&self) -> Arc<dyn Screens> {
        Arc::new(WinScreens)
    }

    /// The system look and the display language.
    #[must_use]
    pub fn look(&self) -> Arc<dyn SystemLook> {
        Arc::new(WinLook)
    }

    /// The window backdrops.
    #[must_use]
    pub fn backdrop(&self) -> Arc<dyn Backdrop> {
        Arc::new(WinBackdrop)
    }

    /// The one running copy of this app in the user session.
    #[must_use]
    pub fn instance(&self) -> Arc<dyn Instance> {
        Arc::new(WinInstance::new(&self.app))
    }

    /// The tray icon in the taskbar's notification area.
    #[must_use]
    pub fn tray(&self) -> Arc<dyn Tray> {
        Arc::new(WinTray::new())
    }

    /// The port that gives the heaps' free pages back to Windows after a look change.
    #[must_use]
    pub fn memory(&self) -> Arc<dyn Memory> {
        Arc::new(WinMemory)
    }

    /// The reveal that shows a folder in Explorer, opened by the running Explorer itself.
    #[must_use]
    pub fn reveal(&self) -> Arc<dyn Reveal> {
        Arc::new(WinReveal)
    }

    /// The hidden-run probe and the hidden copies in each package's private copy of the app-data folder (ADR-0037).
    #[must_use]
    pub fn hidden(&self) -> Arc<dyn Hidden> {
        Arc::new(WinHidden::new())
    }

    /// The speaker: `XAudio2` on a thread of its own, which makes the engine at the first play.
    #[must_use]
    pub fn speaker(&self) -> Arc<dyn Speaker> {
        Arc::new(WinSpeaker::new(self.clock))
    }

    /// The hand's Esc watch: a keyboard hook on its own thread, only while a watch's guard lives (ADR-0059).
    #[must_use]
    pub fn esc(&self) -> Arc<dyn EscWatch> {
        Arc::new(WinEscWatch::new())
    }
}
