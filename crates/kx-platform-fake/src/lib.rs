//! A deterministic fake platform for tests: a clock that moves only when told, and given folders.
//! Only tests, tools and other test-only crates depend on it.

#![forbid(unsafe_code)]

mod clock;

pub use clock::FakeClock;

use kx_module_api::{Clock, Mono};
use kx_platform::{AppDirs, Platform};
use std::sync::Arc;

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
