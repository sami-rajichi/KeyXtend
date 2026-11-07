//! The log: the kernel's redacted subscriber at `[kernel] log_level`, over this run's file in the data folder, capped
//! at `log_max_mb`. When the file cannot open, the run logs to standard error and says so once; a hidden run logs only
//! there.

use kx_kernel::KernelSettings;
use kx_kernel::log::{LogFile, level, max_bytes, subscriber};
use kx_settings::Files;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::Dispatch;
use tracing::level_filters::LevelFilter;

/// Set once the run's log is installed.
static INSTALLED: AtomicBool = AtomicBool::new(false);

/// Where this run logs.
pub struct Log {
    /// The subscriber to install for the whole run.
    pub dispatch: Dispatch,
    /// Why the log file did not open; the run then logs to standard error.
    pub no_file: Option<io::Error>,
}

/// Opens the log `settings` describe, in `files`; `None` when the level names no log level.
#[must_use]
pub fn open(settings: &KernelSettings, files: &Files) -> Option<Log> {
    let level = level(&settings.log_level)?;
    let log = match LogFile::open(files, max_bytes(settings.log_max_mb)) {
        Ok(file) => Log {
            dispatch: Dispatch::new(subscriber(level, file)),
            no_file: None,
        },
        Err(e) => Log {
            no_file: Some(e),
            ..on_stderr(level)
        },
    };
    Some(log)
}

/// The log `settings` describe on standard error alone, for a run that writes no file (ADR-0037); `None` when the level
/// names no log level.
#[must_use]
pub fn unfiled(settings: &KernelSettings) -> Option<Log> {
    level(&settings.log_level).map(on_stderr)
}

/// A log at `level` on standard error.
fn on_stderr(level: LevelFilter) -> Log {
    Log {
        dispatch: Dispatch::new(subscriber(level, io::stderr)),
        no_file: None,
    }
}

/// `loaded`, or the kernel's defaults when no settings loaded.
#[must_use]
pub fn or_defaults(loaded: Option<KernelSettings>) -> Option<KernelSettings> {
    loaded.or_else(|| KernelSettings::from_defaults().ok())
}

/// Makes `log` the whole run's log and says once when it has no file; false when a log was set already.
pub fn install(log: Log) -> bool {
    let Log { dispatch, no_file } = log;
    let set = tracing::dispatcher::set_global_default(dispatch).is_ok();
    INSTALLED.fetch_or(set, Ordering::SeqCst);
    if let Some(e) = no_file {
        tracing::warn!(error = %e, "the log file did not open, so this run logs to standard error");
    }
    set
}

/// True once the run's log is installed.
#[must_use]
pub fn installed() -> bool {
    INSTALLED.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests;
