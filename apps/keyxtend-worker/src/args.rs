//! The command line: `media --max-pixels <n>` runs the media job and `vault` the vault job. The host passes trusted
//! arguments; the voice, OCR and update jobs (ADR-0004) come in later phases, so any other job is refused.

use kx_ipc::{MAX_PIXELS_FLAG, MEDIA_JOB, VAULT_JOB};
use std::ffi::OsString;
use thiserror::Error;

/// What the worker was started to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Job {
    /// Draw images, refusing any that holds more than `max_pixels` pixels.
    Media {
        /// The most pixels an image may hold, at least 1.
        max_pixels: u64,
    },
    /// Make padlocks, seal and open boxes, and read `KeePass` files.
    Vault,
}

/// Why the command line was refused.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ArgError {
    /// No job, or one this worker does not run, as written.
    #[error("unknown job {0:?}; use: {MEDIA_JOB} {MAX_PIXELS_FLAG} <n>, or {VAULT_JOB}")]
    Job(String),
    /// The media job's arguments are missing, extra or not a count above 0.
    #[error("use: {MEDIA_JOB} {MAX_PIXELS_FLAG} <n>, with n above 0")]
    Usage,
    /// The vault job came with arguments.
    #[error("use: {VAULT_JOB}, with nothing after it")]
    Alone,
}

/// The job `args` ask for, without the program's own name.
///
/// # Errors
/// `Job` for no job or an unknown one; `Usage` for a media job without `--max-pixels` and a count above 0 after it;
/// `Alone` for a vault job with anything after it.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Job, ArgError> {
    let mut args = args.into_iter();
    let job = args.next().unwrap_or_default();
    if job == VAULT_JOB {
        return match args.next() {
            None => Ok(Job::Vault),
            Some(_) => Err(ArgError::Alone),
        };
    }
    if job != MEDIA_JOB {
        return Err(ArgError::Job(job.to_string_lossy().into_owned()));
    }
    let (Some(flag), Some(count), None) = (args.next(), args.next(), args.next()) else {
        return Err(ArgError::Usage);
    };
    let max_pixels = count.to_str().and_then(|n| n.parse::<u64>().ok());
    match max_pixels {
        Some(max_pixels) if flag == MAX_PIXELS_FLAG && max_pixels > 0 => {
            Ok(Job::Media { max_pixels })
        }
        _ => Err(ArgError::Usage),
    }
}

#[cfg(test)]
mod tests;
