//! What a dictation, a file's facts or a card's check asks of the `voice` job: the job's limits and times, the model,
//! the card and the run's numbers, each refused outside its range. The `[voice]` settings check against these ranges.

use super::{CardId, SpeechError, TAG_LEN};
use crate::Redacted;
use crate::ports::{fits_iso, path_ok};
use std::ops::RangeInclusive;
use std::path::PathBuf;

/// A worker job's memory cap and waits, shared by every ask of the `voice` and `fetch` jobs.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct JobLimits {
    /// The job's memory cap without a model, in MB; a model adds its own need.
    pub job_mb: u32,
    /// Minutes a job and its model stay after the last run closes.
    pub idle_min: u16,
    /// The longest wait for the worker's answer to an ask, in ms.
    pub ask_ms: u32,
    /// The longest a model may take to load, a fresh voice job to start, or the devices or a file's facts to be read,
    /// in seconds.
    pub load_s: u16,
    /// The longest a card may take to warm up, in seconds.
    pub warm_s: u16,
    /// The longest one piece may take to be written, in seconds.
    pub piece_wait_s: u16,
    /// The wait before a failed worker starts again, in ms; it doubles with each failure in a row.
    pub restart_ms: u32,
    /// The longest wait before a failed worker starts again, in ms.
    pub restart_max_ms: u32,
}

impl JobLimits {
    /// `job_mb`'s range.
    pub const JOB_MB: RangeInclusive<u32> = 64..=4_096;
    /// `idle_min`'s range.
    pub const IDLE_MIN: RangeInclusive<u16> = 1..=120;
    /// `ask_ms`'s range.
    pub const ASK_MS: RangeInclusive<u32> = 500..=30_000;
    /// `load_s`'s range.
    pub const LOAD_S: RangeInclusive<u16> = 5..=600;
    /// `warm_s`'s range.
    pub const WARM_S: RangeInclusive<u16> = 30..=1_800;
    /// `piece_wait_s`'s range.
    pub const PIECE_WAIT_S: RangeInclusive<u16> = 10..=1_800;
    /// `restart_ms`'s range.
    pub const RESTART_MS: RangeInclusive<u32> = 100..=30_000;
    /// `restart_max_ms`'s range.
    pub const RESTART_MAX_MS: RangeInclusive<u32> = 1_000..=600_000;

    /// The name of the first value outside its range, or `restart_ms` when it passes `restart_max_ms`.
    #[must_use]
    pub(crate) fn bad(&self) -> Option<&'static str> {
        let order = (self.restart_ms > self.restart_max_ms).then_some("restart_ms");
        first_bad!(self,
            job_mb: Self::JOB_MB,
            idle_min: Self::IDLE_MIN,
            ask_ms: Self::ASK_MS,
            load_s: Self::LOAD_S,
            warm_s: Self::WARM_S,
            piece_wait_s: Self::PIECE_WAIT_S,
            restart_ms: Self::RESTART_MS,
            restart_max_ms: Self::RESTART_MAX_MS,
        )
        .or(order)
    }

    /// Refuses limits with a value outside its range, or a first wait longer than the longest.
    ///
    /// # Errors
    /// `SpeechError::Range` naming the first such value.
    pub fn check(&self) -> Result<(), SpeechError> {
        refuse(self.bad())
    }
}

/// The model a job loads, as its row or its file says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeechModel {
    /// The model file in the models folder: an absolute drive path (`path_ok`).
    pub file: Redacted<PathBuf>,
    /// The model's own code for the language, such as `fr` or `fr-FR`, within `TAG_LEN`.
    pub lang: String,
    /// The runner that reads it, as the models table names it, within `TAG_LEN`.
    pub runner: String,
    /// The memory it needs on the processor, in MB: its row's, or the size of a file no row names.
    pub cpu_mb: u32,
}

impl SpeechModel {
    /// `cpu_mb`'s range.
    pub const CPU_MB: RangeInclusive<u32> = 1..=65_536;

    /// The name of the first value that breaks its rule, if any.
    fn bad(&self) -> Option<&'static str> {
        let tag = |t: &str| TAG_LEN.contains(&t.len()) && kx_ipc::rules::tag(t).is_ok();
        let shape = [
            (path_ok(self.file.expose()), "file"),
            (tag(&self.lang), "lang"),
            (tag(&self.runner), "runner"),
        ];
        let odd = shape
            .into_iter()
            .find_map(|(fits, name)| (!fits).then_some(name));
        odd.or_else(|| first_bad!(self, cpu_mb: Self::CPU_MB))
    }
}

/// The card that runs a model, and what it must have free for it; the job is asked for their sum.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardUse {
    /// The card.
    pub id: CardId,
    /// The memory the model needs on the card, in MB, from its row.
    pub need_mb: u32,
    /// The room the card must keep free beside it, in MB (`card_margin_mb`).
    pub margin_mb: u32,
}

impl CardUse {
    /// `need_mb`'s range.
    pub const NEED_MB: RangeInclusive<u32> = 1..=65_536;
    /// `margin_mb`'s range.
    pub const MARGIN_MB: RangeInclusive<u32> = 0..=8_192;

    /// The name of the first value outside its range, if any.
    fn bad(&self) -> Option<&'static str> {
        first_bad!(self, need_mb: Self::NEED_MB, margin_mb: Self::MARGIN_MB)
    }
}

/// One dictation: the job, the model and the card, and how the job listens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeechAsk {
    /// The job's limits and waits.
    pub job: JobLimits,
    /// The model that writes the text; `None` listens only, and the sound is handed over at Stop.
    pub model: Option<SpeechModel>,
    /// The card that runs the model; `None` runs it on the processor. A card with no model is refused.
    pub card: Option<CardUse>,
    /// Processor threads; 0 means the real cores, and more are capped at them.
    pub threads: u8,
    /// A piece longer than this, in seconds, is cut at the next pause; a run with no model packs its speech into
    /// pieces no longer than this instead, the cloud row's own cut. Such a piece never passes one frame of sound
    /// (`kx_ipc::MAX_SOUND_BYTES`), so a larger `piece_s` is held to it.
    pub piece_s: u8,
    /// The silence kept around speech when a piece is cut, in ms.
    pub pad_ms: u16,
    /// Levels told per second while listening.
    pub level_hz: u8,
    /// The job stops listening by itself after this many seconds.
    pub record_max_s: u16,
    /// Seconds of silence after speech that stop listening; `None` when that is off.
    pub silence_stop_s: Option<u8>,
}

impl SpeechAsk {
    /// `threads`' range, the frames' own.
    pub const THREADS: RangeInclusive<u8> = kx_ipc::THREADS;
    /// `piece_s`' range, the frames' own.
    pub const PIECE_S: RangeInclusive<u8> = kx_ipc::PIECE_S;
    /// `pad_ms`' range, the frames' own.
    pub const PAD_MS: RangeInclusive<u16> = kx_ipc::PAD_MS;
    /// `level_hz`' range, the frames' own.
    pub const LEVEL_HZ: RangeInclusive<u8> = kx_ipc::LEVEL_HZ;
    /// `record_max_s`' range, the frames' own.
    pub const RECORD_MAX_S: RangeInclusive<u16> = kx_ipc::RECORD_MAX_S;
    /// `silence_stop_s`' range when it is on, the frames' own.
    pub const SILENCE_STOP_S: RangeInclusive<u8> = kx_ipc::SILENCE_STOP_S;

    /// Refuses an ask with a value outside its range, or a card with no model; an ask with no model passes.
    ///
    /// # Errors
    /// `SpeechError::Range` naming the first such value.
    pub fn check(&self) -> Result<(), SpeechError> {
        let lone = (self.card.is_some() && self.model.is_none()).then_some("card");
        let silence = self
            .silence_stop_s
            .is_some_and(|s| !Self::SILENCE_STOP_S.contains(&s));
        let own = || {
            first_bad!(self,
                threads: Self::THREADS,
                piece_s: Self::PIECE_S,
                pad_ms: Self::PAD_MS,
                level_hz: Self::LEVEL_HZ,
                record_max_s: Self::RECORD_MAX_S,
            )
            .or(silence.then_some("silence_stop_s"))
        };
        let bad = (self.job.bad().or(lone).or_else(own))
            .or_else(|| self.model.as_ref().and_then(SpeechModel::bad))
            .or_else(|| self.card.as_ref().and_then(CardUse::bad));
        refuse(bad)
    }
}

/// A card's self-test: the model on the shipped clip, on the card and on the processor; the voice module scores it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckAsk {
    /// The job's limits and waits.
    pub job: JobLimits,
    /// The model under test.
    pub model: SpeechModel,
    /// The card under test.
    pub card: CardUse,
    /// Processor threads for the processor's run, as `SpeechAsk::threads`.
    pub threads: u8,
    /// The shipped clip's language, an ISO 639 code such as `ar`: the language in front's when the model knows it,
    /// else the first shipped clip it knows (ADR-0052).
    pub clip: String,
}

impl CheckAsk {
    /// Refuses a check with a value outside its range, or a clip's language that is no ISO 639 code.
    ///
    /// # Errors
    /// `SpeechError::Range` naming the first such value.
    pub fn check(&self) -> Result<(), SpeechError> {
        let clip = (!fits_iso(&self.clip)).then_some("clip");
        let threads = || first_bad!(self, threads: SpeechAsk::THREADS);
        let bad = (self.job.bad().or_else(|| self.model.bad()))
            .or_else(|| self.card.bad())
            .or(clip)
            .or_else(threads);
        refuse(bad)
    }
}

/// A model file whose facts are read, such as one no row names, and the job that reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactsAsk {
    /// The job's limits and waits.
    pub job: JobLimits,
    /// The file: an absolute drive path (`path_ok`).
    pub file: Redacted<PathBuf>,
    /// Its size in MB, within `SpeechModel::CPU_MB`; the job is capped at it plus `job_mb`.
    pub size_mb: u32,
}

impl FactsAsk {
    /// Refuses an ask with a job value outside its range, a file that is no plain drive path, or a size outside
    /// `SpeechModel::CPU_MB`.
    ///
    /// # Errors
    /// `SpeechError::BadPath` for the file, else `SpeechError::Range` naming the value.
    pub fn check(&self) -> Result<(), SpeechError> {
        refuse(self.job.bad())?;
        if !path_ok(self.file.expose()) {
            return Err(SpeechError::BadPath);
        }
        refuse(first_bad!(self, size_mb: SpeechModel::CPU_MB))
    }
}

/// `Range` naming `bad`, or fine when there is none.
fn refuse(bad: Option<&'static str>) -> Result<(), SpeechError> {
    bad.map_or(Ok(()), |name| Err(SpeechError::Range(name)))
}
