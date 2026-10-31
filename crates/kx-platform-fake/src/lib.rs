//! A deterministic fake platform for tests: a clock that moves only when told, given folders, and recording fakes
//! of the platform's ports, from input, layouts and the UI to the clipboard, the sealed shelf, pictures and the
//! calendar. Only tests, tools and test-only crates use it.

#![forbid(unsafe_code)]

mod calendar;
mod clip;
mod clock;
mod field;
mod input;
mod instance;
mod layout;
mod layout_file;
mod layouts;
mod look;
mod memory;
mod packs;
mod pictures;
mod pointer;
mod power;
mod reveal;
mod scroll;
mod sealed;
mod shelf;
mod shortcut;
mod sound;
mod tray;
mod ui;

pub use calendar::{FakeCalendar, PARIS};
pub use clip::{FakeClipboard, FakeClipboardWriter};
pub use clock::FakeClock;
pub use field::FakeFieldWatch;
pub use input::{FakeInjector, Stroke};
pub use instance::FakeInstance;
pub use layout::FakeLayoutSource;
pub use layout_file::{FILE_EXT, LayoutFile, LayoutFileError, golden_dir};
pub use layouts::{Ask, FakeLayouts};
pub use look::{DEFAULT_LANGUAGE, FakeSystemLook};
pub use memory::FakeMemory;
pub use packs::FakeWordPacks;
pub use pictures::FakePictures;
pub use pointer::FakePointerHook;
pub use power::FakePowerDialog;
pub use reveal::FakeReveal;
pub use scroll::{FakeScrollTarget, Turn};
pub use sealed::FakeSealedStore;
pub use shelf::{FakeShelf, ShelfCall};
pub use shortcut::FakeShortcuts;
pub use sound::{FakeSpeaker, Prepared};
pub use tray::FakeTray;
pub use ui::{FakeBackdrop, FakeScreens, FakeWindowGuard, GuardCall};

use kx_module_api::{Clock, Mono};
use kx_platform::{AppDirs, Platform};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Locks `m`, taking the value even when a panicking test poisoned it.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A `Platform` built from a `FakeClock` and folders the test names.
#[derive(Debug)]
pub struct FakePlatform {
    clock: FakeClock,
    dirs: AppDirs,
}

impl FakePlatform {
    /// A platform whose clock starts at zero and whose folders are `dirs`.
    #[must_use]
    pub fn new(dirs: AppDirs) -> Self {
        Self {
            clock: FakeClock::new(Mono::default()),
            dirs,
        }
    }

    /// A handle to the platform's clock, for the test to advance.
    #[must_use]
    pub fn clock_handle(&self) -> FakeClock {
        self.clock.clone()
    }
}

impl Platform for FakePlatform {
    fn clock(&self) -> Arc<dyn Clock> {
        Arc::new(self.clock.clone())
    }

    fn dirs(&self) -> &AppDirs {
        &self.dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    fn dirs() -> AppDirs {
        AppDirs {
            exe_dir: PathBuf::from("app"),
            user_dir: PathBuf::from("user"),
        }
    }

    #[test]
    fn the_clock_sees_the_time_the_test_handle_sets() {
        let platform = FakePlatform::new(dirs());
        let clock = platform.clock();
        assert_eq!(clock.now(), Mono::default());
        platform.clock_handle().advance(Duration::from_micros(30));
        assert_eq!(clock.now(), Mono::from_us(30));
        platform.clock_handle().set(Mono::from_us(9));
        assert_eq!(clock.now(), Mono::from_us(9));
    }

    #[test]
    fn the_dirs_are_the_ones_given() {
        assert_eq!(FakePlatform::new(dirs()).dirs(), &dirs());
    }

    #[test]
    fn it_works_as_a_platform_object() {
        let platform: Arc<dyn Platform> = Arc::new(FakePlatform::new(dirs()));
        assert_eq!(platform.dirs(), &dirs());
        assert_eq!(platform.clock().now(), Mono::default());
    }
}
