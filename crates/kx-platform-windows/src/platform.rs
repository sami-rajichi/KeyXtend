//! `WindowsPlatform`: the clock and folders the kernel gets on Windows, and the ports the app registers.

use std::path::PathBuf;
use std::sync::Arc;

use kx_crypto::KeyGuard;
use kx_module_api::Clock;
use kx_module_api::ports::{
    FieldWatch, InputInjector, LayoutSource, PointerHook, PowerDialog, ScrollTarget, Shortcuts,
    WordPacks,
};
use kx_platform::ui::{
    Backdrop, Instance, Memory, Reveal, Screens, Speaker, SystemLook, Tray, WindowGuard,
};
use kx_platform::{APP_DIR_NAME, AppDirs, Platform};

use crate::hook::Source;
use crate::{
    PlatformError, QpcClock, WinBackdrop, WinFieldWatch, WinGuard, WinInjector, WinInstance,
    WinKeyGuard, WinLayouts, WinLook, WinMemory, WinPackDisk, WinPointerHook, WinPower, WinReveal,
    WinScreens, WinScroll, WinShortcuts, WinSpeaker, WinTray, WinWordPacks, dirs,
};

/// The Windows `Platform`: the QPC clock and the program and per-user folders.
#[derive(Debug)]
pub struct WindowsPlatform {
    clock: QpcClock,
    dirs: AppDirs,
    /// Names the running copy's mutex and command window.
    app: String,
}

impl WindowsPlatform {
    /// Reads the clock frequency and the folders once.
    ///
    /// # Errors
    /// `PlatformError` when Windows refuses the clock or a folder.
    pub fn new() -> Result<Self, PlatformError> {
        Self::named(APP_DIR_NAME)
    }

    /// The platform of app `app`, so a test's running copy never meets a real one.
    pub(crate) fn named(app: &str) -> Result<Self, PlatformError> {
        Ok(Self {
            clock: QpcClock::new()?,
            dirs: dirs::current()?,
            app: app.to_owned(),
        })
    }

    /// The input injector, for the app to register under `InputInjectorKey`.
    #[must_use]
    pub fn injector(&self) -> Arc<dyn InputInjector> {
        Arc::new(WinInjector)
    }

    /// The layout source, for the app to register under `LayoutSourceKey`.
    #[must_use]
    pub fn layouts(&self) -> Arc<dyn LayoutSource> {
        Arc::new(WinLayouts)
    }

    /// The shortcuts service, for the app to register under `ShortcutsKey`.
    #[must_use]
    pub fn shortcuts(&self) -> Arc<dyn Shortcuts> {
        Arc::new(WinShortcuts)
    }

    /// The system's power dialog, for the app to register under `PowerDialogKey`.
    #[must_use]
    pub fn power_dialog(&self) -> Arc<dyn PowerDialog> {
        Arc::new(WinPower)
    }

    /// The data key's guard for the app's sealed files: DPAPI for the signed-in user.
    #[must_use]
    pub fn key_guard(&self) -> Box<dyn KeyGuard> {
        Box::new(WinKeyGuard)
    }

    /// The word packs in `dirs`, the first folder first, for the app to register under `WordPacksKey`.
    #[must_use]
    pub fn word_packs(&self, dirs: Vec<PathBuf>) -> Arc<dyn WordPacks> {
        Arc::new(WinWordPacks::new(WinPackDisk, dirs))
    }

    /// The focused-box watch over UI Automation, for the app to register under `FieldWatchKey`.
    #[must_use]
    pub fn field_watch(&self) -> Arc<dyn FieldWatch> {
        Arc::new(WinFieldWatch)
    }

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

    /// The speaker: `XAudio2` on a thread of its own, which makes the engine at the first play.
    #[must_use]
    pub fn speaker(&self) -> Arc<dyn Speaker> {
        Arc::new(WinSpeaker::new(self.clock))
    }

    /// The scroll target, for the app to register under `ScrollTargetKey`: each run reads the point the guard keeps
    /// at the keyboard's entries.
    #[must_use]
    pub fn scroll_target(&self) -> Arc<dyn ScrollTarget> {
        Arc::new(WinScroll::new(self.clock))
    }

    /// The mouse hook on the hand's input, for the app to register under `PointerHookKey`.
    #[must_use]
    pub fn pointer_hook(&self) -> Arc<dyn PointerHook> {
        Arc::new(WinPointerHook::new(self.clock, Source::Physical))
    }

    /// The mouse hook on the gate tool's simulated input instead of the hand's; only with the `sim-input` feature.
    #[cfg(feature = "sim-input")]
    #[must_use]
    pub fn sim_pointer_hook(&self) -> Arc<dyn PointerHook> {
        Arc::new(WinPointerHook::new(self.clock, Source::Simulated))
    }
}

impl Platform for WindowsPlatform {
    fn clock(&self) -> Arc<dyn Clock> {
        Arc::new(self.clock)
    }

    fn dirs(&self) -> &AppDirs {
        &self.dirs
    }
}

#[cfg(test)]
mod tests;
