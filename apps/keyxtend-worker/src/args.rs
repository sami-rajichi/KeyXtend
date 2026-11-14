//! The command line: `media --max-pixels <n>` runs the media job, and `vault` and `voice` their jobs, alone. The host
//! passes trusted arguments; the fetch, OCR and update jobs come in later tasks and phases, so any other job is refused.

use kx_ipc::{MAX_PIXELS_FLAG, MEDIA_JOB, VAULT_JOB, VOICE_JOB};
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
    /// Record the microphone, cut it into pieces and hand its sound over, with no network (ADR-0042).
    Voice,
}

/// Why the command line was refused.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ArgError {
    /// No job, or one this worker does not run, as written.
    #[error(
        "unknown job {0:?}; use: {MEDIA_JOB} {MAX_PIXELS_FLAG} <n>, {VAULT_JOB} or {VOICE_JOB}"
    )]
    Job(String),
    /// The media job's arguments are missing, extra or not a count above 0.
    #[error("use: {MEDIA_JOB} {MAX_PIXELS_FLAG} <n>, with n above 0")]
    Usage,
    /// A job that takes no argument came with some; it names that job.
    #[error("use: {0}, with nothing after it")]
    Alone(&'static str),
}

/// The job `args` ask for, without the program's own name.
///
/// # Errors
/// `Job` for no job or an unknown one; `Usage` for a media job without `--max-pixels` and a count above 0 after it;
/// `Alone` for a vault or voice job with anything after it.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Job, ArgError> {
    let mut args = args.into_iter();
    let job = args.next().unwrap_or_default();
    let alone = if job == VAULT_JOB {
        Some((VAULT_JOB, Job::Vault))
    } else if job == VOICE_JOB {
        Some((VOICE_JOB, Job::Voice))
    } else {
        None
    };
    if let Some((word, job)) = alone {
        return match args.next() {
            None => Ok(job),
            Some(_) => Err(ArgError::Alone(word)),
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
