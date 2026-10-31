//! The worker's command words and exit codes, read by the host that starts it and by the worker itself.

/// The job that draws pictures, the first argument.
pub const MEDIA_JOB: &str = "media";
/// The flag before the most pixels an image may hold.
pub const MAX_PIXELS_FLAG: &str = "--max-pixels";
/// The exit code after goodbye or the end of input; every OS reads 0 as success.
pub const CLEAN_EXIT: u8 = 0;
/// The exit code after a refused command line or a broken frame.
pub const REFUSED_EXIT: u8 = 1;
