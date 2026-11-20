//! Test-only: what the app's log file holds once a closure has logged on this thread, and the values of a gate line's
//! field, read as kx-gates reads them.

use kx_kernel::KernelSettings;
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;

/// What this run's log file holds at `level`, or the kernel's default level, once `f` has logged on this thread.
pub(crate) fn at(level: Option<&str>, f: impl FnOnce()) -> String {
    let dir = TempDir::new("played-logged").unwrap();
    let files = Files::new(dir.path());
    let mut settings = KernelSettings::from_defaults().unwrap();
    if let Some(level) = level {
        level.clone_into(&mut settings.log_level);
    }
    let log = crate::log::open(&settings, &files).unwrap();
    tracing::dispatcher::with_default(&log.dispatch, f);
    std::fs::read_to_string(&files.log).unwrap()
}

/// The values of field `name` on `text`'s lines holding `marker`, in order.
pub(crate) fn values(text: &str, marker: &str, name: &str) -> Vec<String> {
    let value = |l: &str| {
        let words = l.split_whitespace();
        let word = words
            .filter_map(|w| w.strip_prefix(name))
            .find_map(|w| w.strip_prefix('='));
        word.map(str::to_owned)
    };
    text.lines()
        .filter(|l| l.contains(marker))
        .filter_map(value)
        .collect()
}
