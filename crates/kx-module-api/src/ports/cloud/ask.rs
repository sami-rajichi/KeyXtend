//! What a cloud run or a key's test asks of `speech-net`: the job, the row's request and limits, the key and the
//! pieces, each refused outside its range. The `[voice]` settings and the services table check against these ranges.

use super::{CloudFault, CloudShape, HttpsUrl, KeyHeader};
use crate::ports::{JobLimits, SAMPLE_BYTES, TAG_LEN};
use crate::{Redacted, Secret, SecretBytes};
pub use kx_ipc::MAX_KEY_BYTES;
use std::ops::RangeInclusive;

/// A service row's limits on one request, from the service's own pages; a dictation is cut within them.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CloudLimits {
    /// The most megabytes of raw sound one request may carry: the table stores the service's cap already reduced for
    /// its shape's growth (Base64, the WAV header, the form's parts), as `piece_bytes` counts raw sound.
    pub max_mb: u32,
    /// The most seconds of sound one request may carry.
    pub max_s: u16,
    /// The most requests a minute.
    pub per_min: u16,
    /// The most requests at once.
    pub at_once: u8,
}

impl CloudLimits {
    /// `max_mb`'s range, the frames' own.
    pub const MAX_MB: RangeInclusive<u32> = kx_ipc::NET_MAX_MB;
    /// `max_s`' range: from the shortest piece the `voice` job cuts to the longest dictation, the frames' own.
    pub const MAX_S: RangeInclusive<u16> = kx_ipc::NET_MAX_S;
    /// `per_min`'s range, the frames' own.
    pub const PER_MIN: RangeInclusive<u16> = kx_ipc::NET_PER_MIN;
    /// `at_once`' range, the frames' own.
    pub const AT_ONCE: RangeInclusive<u8> = kx_ipc::NET_AT_ONCE;

    /// The most bytes one piece may hold: within both `max_mb` and `max_s`, in whole samples, as the frames check.
    #[must_use]
    pub fn piece_bytes(&self) -> usize {
        kx_ipc::piece_bytes(self.max_mb, self.max_s)
    }

    /// The name of the first value outside its range, if any.
    fn bad(&self) -> Option<&'static str> {
        first_bad!(self,
            max_mb: Self::MAX_MB,
            max_s: Self::MAX_S,
            per_min: Self::PER_MIN,
            at_once: Self::AT_ONCE,
        )
    }
}

/// One dictation's pieces for one service, as its row and the user's choices say.
#[derive(Debug, PartialEq, Eq)]
pub struct CloudAsk {
    /// The `speech-net` job's limits and waits; its cap is `job_mb`.
    pub job: JobLimits,
    /// The request's shape, with a form's field names.
    pub shape: CloudShape,
    /// Where the pieces go: the row's address, or the one the user typed.
    pub url: HttpsUrl,
    /// The header that carries the key.
    pub header: KeyHeader,
    /// The user's key: visible ASCII within `MAX_KEY_BYTES`, wiped on drop.
    pub key: Secret,
    /// The model's id, such as `openai/whisper-large-v3` (`rules::model`), redacted as a typed Other model is the
    /// user's; `None` sends none, as Lemonfox and Azure's classic take none, and `needs_model` shapes refuse it.
    pub model: Option<Redacted<String>>,
    /// The model's code for the language in front, such as `fr`, `ar-SA` or `french`: a `rules::token` in `TAG_LEN`.
    pub lang: String,
    /// The row's limits.
    pub limits: CloudLimits,
    /// The longest wait for one request's answer, in seconds (`net_wait_s`).
    pub wait_s: u16,
    /// Tries per piece before its fault is told (`net_tries`).
    pub tries: u8,
    /// The pieces in order, as the `voice` job cut them: 16 kHz mono 16-bit sound, each within `limits.piece_bytes`, at
    /// most `MAX_RECORD_BYTES` together.
    pub pieces: Vec<SecretBytes>,
}

impl CloudAsk {
    /// `wait_s`' range, the frames' own; the default is the `[voice]` settings'.
    pub const WAIT_S: RangeInclusive<u16> = kx_ipc::NET_WAIT_S;
    /// `tries`' range, the frames' own; the default is the `[voice]` settings'.
    pub const TRIES: RangeInclusive<u8> = kx_ipc::NET_TRIES;
    /// How many pieces an ask holds: from one to the most a listen-only Stop hands over, the frames' own.
    pub const PIECES: RangeInclusive<usize> = 1..=kx_ipc::MAX_SPEECH_PIECES;

    /// Refuses an ask with a value outside its range or one that could break a request.
    ///
    /// # Errors
    /// `CloudFault::Range` naming the first such value.
    pub fn check(&self) -> Result<(), CloudFault> {
        let model = self
            .model
            .as_ref()
            .map_or(!self.shape.code().needs_model(), |m| {
                fits(m.expose(), kx_ipc::rules::model)
            });
        let codes = [
            (model, "model"),
            (fits(&self.lang, kx_ipc::rules::token), "lang"),
        ];
        let bad = self
            .job
            .bad()
            .or_else(|| head_bad(&self.shape, &self.header, &self.key))
            .or_else(|| first_odd(codes))
            .or_else(|| self.limits.bad())
            .or_else(|| first_bad!(self, wait_s: Self::WAIT_S, tries: Self::TRIES))
            .or((!self.pieces_fit()).then_some("pieces"));
        refuse(bad)
    }

    /// True for one to `PIECES` pieces, each whole samples within `limits.piece_bytes`, at most `MAX_RECORD_BYTES`
    /// together, as the frames check.
    fn pieces_fit(&self) -> bool {
        let most = self.limits.piece_bytes();
        let mut sizes = self.pieces.iter().map(|p| p.expose().len());
        let total: usize = sizes.clone().sum();
        let count = Self::PIECES.contains(&self.pieces.len());
        let piece = |n: usize| n > 0 && n <= most && n.is_multiple_of(SAMPLE_BYTES);
        count && total <= kx_ipc::MAX_RECORD_BYTES && sizes.all(piece)
    }
}

/// A key's test: a call to the row's test address that needs the key and sends no sound.
#[derive(Debug, PartialEq, Eq)]
pub struct CloudTest {
    /// The `speech-net` job's limits and waits.
    pub job: JobLimits,
    /// The row's shape, whose adapter makes the call and reads its answer.
    pub shape: CloudShape,
    /// The row's test address, or one under the address the user typed.
    pub url: HttpsUrl,
    /// The header that carries the key.
    pub header: KeyHeader,
    /// The key to test, as `CloudAsk::key`.
    pub key: Secret,
    /// The longest wait for the answer, in seconds, within `CloudAsk::WAIT_S`.
    pub wait_s: u16,
}

impl CloudTest {
    /// Refuses a test with a value outside its range or one that could break a request.
    ///
    /// # Errors
    /// `CloudFault::Range` naming the first such value.
    pub fn check(&self) -> Result<(), CloudFault> {
        let bad = self
            .job
            .bad()
            .or_else(|| head_bad(&self.shape, &self.header, &self.key))
            .or_else(|| first_bad!(self, wait_s: CloudAsk::WAIT_S));
        refuse(bad)
    }
}

/// The name of the first of a request's shape, header and key that could break it, if any.
fn head_bad(shape: &CloudShape, header: &KeyHeader, key: &Secret) -> Option<&'static str> {
    let key = key.expose().as_bytes();
    let key_ok = (1..=MAX_KEY_BYTES).contains(&key.len()) && kx_ipc::rules::key(key).is_ok();
    first_odd([
        (shape.fits(), "shape"),
        (header.fits(), "header"),
        (key_ok, "key"),
    ])
}

/// True for a code within `TAG_LEN` that passes `rule`, as the frames take a model's id or a language code.
fn fits(text: &str, rule: fn(&str) -> Result<(), kx_ipc::IpcError>) -> bool {
    TAG_LEN.contains(&text.len()) && rule(text).is_ok()
}

/// The name of the first rule that does not hold, if any.
fn first_odd<const N: usize>(rules: [(bool, &'static str); N]) -> Option<&'static str> {
    rules
        .into_iter()
        .find_map(|(fits, name)| (!fits).then_some(name))
}

/// `Range` naming `bad`, or fine when there is none.
fn refuse(bad: Option<&'static str>) -> Result<(), CloudFault> {
    bad.map_or(Ok(()), |name| Err(CloudFault::Range(name)))
}
