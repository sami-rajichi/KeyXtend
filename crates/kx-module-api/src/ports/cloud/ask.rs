//! What a cloud run or a key's test asks of `speech-net`: the job, the row's request and limits, the key and the
//! pieces, each refused outside its range. The `[voice]` settings and the services table check against these ranges.

use super::{CloudFault, CloudShape, HttpsUrl, KeyHeader};
use crate::ports::{JobLimits, MB, SAMPLE_BYTES, SOUND_HZ, TAG_LEN};
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
    pub const MAX_S: RangeInclusive<u16> =
        RangeInclusive::new(*kx_ipc::PIECE_S.start() as u16, *kx_ipc::RECORD_MAX_S.end());
    /// `per_min`'s range, the frames' own.
    pub const PER_MIN: RangeInclusive<u16> = kx_ipc::NET_PER_MIN;
    /// `at_once`' range, the frames' own.
    pub const AT_ONCE: RangeInclusive<u8> = kx_ipc::NET_AT_ONCE;

    /// The most bytes one piece may hold: within both `max_mb` and `max_s`, in whole samples.
    #[must_use]
    pub fn piece_bytes(&self) -> usize {
        let by_size = u64::from(self.max_mb).saturating_mul(MB);
        let by_size = usize::try_from(by_size).unwrap_or(usize::MAX);
        let by_time = usize::from(self.max_s) * SOUND_HZ * SAMPLE_BYTES;
        let most = by_size.min(by_time);
        most - most % SAMPLE_BYTES
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
    /// The model's id, such as `whisper-large-v3-turbo`, redacted as a typed Other model is the user's; `None` sends
    /// none, as Lemonfox and Azure's classic take none.
    pub model: Option<Redacted<String>>,
    /// The model's code for the language in front, such as `fr`, `ar-SA` or `french`, within `TAG_LEN`.
    pub lang: String,
    /// The row's limits.
    pub limits: CloudLimits,
    /// The longest wait for one request's answer, in seconds (`net_wait_s`).
    pub wait_s: u16,
    /// Tries per piece before its fault is told (`net_tries`).
    pub tries: u8,
    /// The pieces in order, as the `voice` job cut them: 16 kHz mono 16-bit sound, each within `limits.piece_bytes`.
    pub pieces: Vec<SecretBytes>,
}

impl CloudAsk {
    /// `wait_s`' range, the frames' own; the default is the `[voice]` settings'.
    pub const WAIT_S: RangeInclusive<u16> = kx_ipc::NET_WAIT_S;
    /// `tries`' range, the frames' own; the default is the `[voice]` settings'.
    pub const TRIES: RangeInclusive<u8> = kx_ipc::NET_TRIES;
    /// How many pieces an ask holds: from one to a whole dictation's, the frames' own.
    pub const PIECES: RangeInclusive<usize> = 1..=kx_ipc::MAX_PIECES;

    /// Refuses an ask with a value outside its range or one that could break a request.
    ///
    /// # Errors
    /// `CloudFault::Range` naming the first such value.
    pub fn check(&self) -> Result<(), CloudFault> {
        let most = self.limits.piece_bytes();
        let piece = |p: &SecretBytes| {
            let n = p.expose().len();
            n > 0 && n <= most && n.is_multiple_of(SAMPLE_BYTES)
        };
        let codes = [
            (
                self.model.as_ref().is_none_or(|m| tag_ok(m.expose())),
                "model",
            ),
            (tag_ok(&self.lang), "lang"),
        ];
        let pieces = Self::PIECES.contains(&self.pieces.len()) && self.pieces.iter().all(piece);
        let bad = self
            .job
            .bad()
            .or_else(|| head_bad(&self.shape, &self.header, &self.key))
            .or_else(|| first_odd(codes))
            .or_else(|| self.limits.bad())
            .or_else(|| first_bad!(self, wait_s: Self::WAIT_S, tries: Self::TRIES))
            .or((!pieces).then_some("pieces"));
        refuse(bad)
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
    let key = key.expose();
    let key_ok =
        (1..=MAX_KEY_BYTES).contains(&key.len()) && key.bytes().all(|b| b.is_ascii_graphic());
    first_odd([
        (shape.fits(), "shape"),
        (header.fits(), "header"),
        (key_ok, "key"),
    ])
}

/// True for a plain tag within `TAG_LEN`, as the frames take a language code.
fn tag_ok(text: &str) -> bool {
    TAG_LEN.contains(&text.len()) && kx_ipc::rules::tag(text).is_ok()
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
