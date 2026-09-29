//! `WindowsPlatform`: the clock and folders the kernel gets on Windows, and the ports the app registers.

use std::sync::Arc;

use kx_module_api::Clock;
use kx_module_api::ports::{InputInjector, LayoutSource};
use kx_platform::{AppDirs, Platform};

use crate::{PlatformError, QpcClock, WinInjector, WinLayouts, dirs};

/// The Windows `Platform`: the QPC clock and the program and per-user folders.
#[derive(Debug)]
pub struct WindowsPlatform {
    clock: QpcClock,
    dirs: AppDirs,
}

impl WindowsPlatform {
    /// Reads the clock frequency and the folders once.
    ///
    /// # Errors
    /// `PlatformError` when Windows refuses the clock or a folder.
    pub fn new() -> Result<Self, PlatformError> {
        Ok(Self {
            clock: QpcClock::new()?,
            dirs: dirs::current()?,
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
    fn the_layout_source_is_a_port_that_lists_the_installed_layouts() {
        let layouts = WindowsPlatform::new().unwrap().layouts();
        let list = std::thread::spawn(move || layouts.installed())
            .join()
            .unwrap();
        assert!(!list.is_empty());
    }
}
