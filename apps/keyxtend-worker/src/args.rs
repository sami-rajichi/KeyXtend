//! The command line: `media --max-pixels <n>` runs the media job, and `vault`, `voice`, `fetch`, `speech-net`, `record`
//! and `pick` their jobs, alone. The host passes trusted arguments and only a gate build takes `voice --sound <file.wav>`;
//! any other job is refused.

#[cfg(feature = "gate")]
use kx_ipc::SOUND_FLAG;
use kx_ipc::{
    FETCH_JOB, MAX_PIXELS_FLAG, MEDIA_JOB, NET_JOB, PICK_JOB, RECORD_JOB, VAULT_JOB, VOICE_JOB,
};
use std::ffi::OsString;
#[cfg(feature = "gate")]
use std::path::PathBuf;
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
    /// Download model files over HTTPS, with no microphone (ADR-0042).
    Fetch,
    /// Send a dictation's pieces to the cloud service the user switched on, with no microphone (ADR-0047).
    SpeechNet,
    /// Record a part of the screen with its sounds into a video, and write re-timed copies, with no network (ADR-0056).
    Record,
    /// Gate builds only: the voice job with this WAV file in place of the microphone.
    #[cfg(feature = "gate")]
    Played(PathBuf),
    /// Show the system's Save and folder dialogs, with no network (ADR-0056).
    Pick,
}

/// Why the command line was refused.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum ArgError {
    /// No job, or one this worker does not run, as written.
    #[error(
        "unknown job {0:?}; use: {MEDIA_JOB} {MAX_PIXELS_FLAG} <n>, {VAULT_JOB}, {VOICE_JOB}, {FETCH_JOB}, {NET_JOB}, \
         {RECORD_JOB} or {PICK_JOB}"
    )]
    Job(String),
    /// The media job's arguments are missing, extra or not a count above 0.
    #[error("use: {MEDIA_JOB} {MAX_PIXELS_FLAG} <n>, with n above 0")]
    Usage,
    /// A job that takes no argument came with some; it names that job.
    #[error("use: {0}, with nothing after it")]
    Alone(&'static str),
    /// Gate builds only: the sound flag without exactly one file after it.
    #[cfg(feature = "gate")]
    #[error("use: {VOICE_JOB} {SOUND_FLAG} <file.wav>")]
    Sound,
}

/// The job `args` ask for, without the program's own name.
///
/// # Errors
/// `Job` for no job or an unknown one; `Usage` for a media job without `--max-pixels` and a count above 0 after it;
/// `Alone` for a vault, voice, fetch, speech, record or pick job with anything after it.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Job, ArgError> {
    let mut args = args.into_iter();
    let job = args.next().unwrap_or_default();
    let alone = [
        (VAULT_JOB, Job::Vault),
        (VOICE_JOB, Job::Voice),
        (FETCH_JOB, Job::Fetch),
        (NET_JOB, Job::SpeechNet),
        (RECORD_JOB, Job::Record),
        (PICK_JOB, Job::Pick),
    ]
    .into_iter()
    .find(|(word, _)| job == *word);
    if let Some((word, job)) = alone {
        return match args.next() {
            None => Ok(job),
            #[cfg(feature = "gate")]
            Some(flag) if job == Job::Voice && flag == SOUND_FLAG => played(args),
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

/// The voice job playing the one file left in `args`.
#[cfg(feature = "gate")]
fn played(mut args: impl Iterator<Item = OsString>) -> Result<Job, ArgError> {
    match (args.next(), args.next()) {
        (Some(file), None) => Ok(Job::Played(PathBuf::from(file))),
        _ => Err(ArgError::Sound),
    }
}

#[cfg(test)]
mod tests;
