//! `WindowsPlatform`: the clock and folders the kernel gets on Windows, and the ports the app registers.

use std::sync::Arc;

use kx_module_api::Clock;
use kx_module_api::ports::{InputInjector, LayoutSource};
use kx_platform::ui::{Backdrop, Instance, Screens, SystemLook, Tray, WindowGuard};
use kx_platform::{APP_DIR_NAME, AppDirs, Platform};

use crate::{
    PlatformError, QpcClock, WinBackdrop, WinGuard, WinInjector, WinInstance, WinLayouts, WinLook,
    WinScreens, WinTray, dirs,
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
mod tests {
    use super::*;
    use kx_module_api::Mono;

    #[test]
    fn the_dirs_are_the_known_folder_ones() {
        let platform = WindowsPlatform::new().unwrap();
        assert_eq!(platform.dirs(), &dirs::current().unwrap());
    }

    #[test]
    fn it_works_as_a_platform_object_with_a_running_clock() {
        let platform: Arc<dyn Platform> = Arc::new(WindowsPlatform::new().unwrap());
        let clock = platform.clock();
        let first = clock.now();
        assert!(first > Mono::default() && clock.now() >= first);
    }

    #[test]
    fn the_injector_is_a_port_other_threads_can_use() {
        let injector = WindowsPlatform::new().unwrap().injector();
        let asked = std::thread::spawn(move || injector.target_app()).join();
        assert!(asked.is_ok(), "the port answers on another thread");
    }

    #[test]
    fn the_ui_ports_answer_on_another_thread() {
        let platform = WindowsPlatform::new().unwrap();
        let (screens, look, guard) = (platform.screens(), platform.look(), platform.guard());
        let answers = std::thread::spawn(move || {
            (
                screens.work_areas(),
                look.ui_language(),
                guard.guard_own(None, None),
            )
        });
        let (areas, language, guarded) = answers.join().unwrap();
        assert!(!areas.is_empty() && !language.is_empty());
        assert_eq!(guarded, Ok(0), "no window of ours is shown");
    }

    #[test]
    fn the_tray_port_names_its_icon_size_on_another_thread() {
        let tray = WindowsPlatform::new().unwrap().tray();
        let px = std::thread::spawn(move || tray.icon_px()).join().unwrap();
        assert!(px > 0);
    }

    #[test]
    fn the_backdrop_port_finds_no_window_we_do_not_show() {
        use kx_platform::ui::{BackdropKind, UiError};
        let platform = WindowsPlatform::new().unwrap();
        let missing = platform
            .backdrop()
            .apply("KeyXtend test: none", BackdropKind::None, false);
        assert_eq!(missing, Err(UiError::NoWindow));
    }

    #[test]
    fn two_platforms_meet_on_one_running_copy() {
        use kx_platform::ui::Running;
        // A test name, so a real copy starting now never reads this test as running.
        let app = format!("KeyXtendTest.{}.platform", std::process::id());
        let first = WindowsPlatform::named(&app).unwrap().instance();
        let second = WindowsPlatform::named(&app).unwrap().instance();
        let claim = first.claim().unwrap();
        assert!(matches!(second.claim(), Err(Running)), "one session name");
        drop(claim);
        assert!(second.claim().is_ok(), "free again once dropped");
        assert_eq!(
            WindowsPlatform::new().unwrap().app,
            APP_DIR_NAME,
            "the app's own name"
        );
    }

    #[test]
    fn the_layout_source_is_a_port_that_lists_the_installed_layouts() {
        let layouts = WindowsPlatform::new().unwrap().layouts();
        let list = std::thread::spawn(move || layouts.installed())
            .join()
            .unwrap();
        assert!(!list.is_empty());
    }
}
