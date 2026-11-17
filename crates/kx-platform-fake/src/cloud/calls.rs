//! What the fake cloud records: each run's ask without its key or sound, with each piece's size and where it stands,
//! and each key test's ask without its key, with its answer.

use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudLimits, CloudNote, CloudShape, CloudTest, HttpsUrl, JobLimits,
    KeyHeader,
};
use kx_module_api::{Redacted, Secret, Zeroizing};

/// A piece's scripted answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloudAnswer {
    /// The piece's text.
    Text(Secret),
    /// The service's fault on the piece; the run stops on it.
    Fault(CloudFault),
}

impl CloudAnswer {
    /// `text` as a piece's answer.
    #[must_use]
    pub fn text(text: &str) -> Self {
        Self::Text(Redacted::new(Zeroizing::new(text.to_owned())))
    }
}

/// Where a run stands.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CloudState {
    /// Some pieces wait for their answer.
    Sending,
    /// Every piece's text was told, then `Done`.
    Done,
    /// It stopped on this fault: scripted, cut, or the key the service refused.
    Fault(CloudFault),
    /// It was dropped while sending, which cancelled its unanswered pieces.
    Dropped,
}

/// One run: its ask without the key or the sound, each piece's size, and where it stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudLog {
    /// The job's limits.
    pub job: JobLimits,
    /// The request's shape.
    pub shape: CloudShape,
    /// Where the pieces went.
    pub url: HttpsUrl,
    /// The header that carried the key.
    pub header: KeyHeader,
    /// The model's id, redacted.
    pub model: Option<Redacted<String>>,
    /// The model's language code.
    pub lang: String,
    /// The row's limits.
    pub limits: CloudLimits,
    /// The longest wait for one answer, in seconds.
    pub wait_s: u16,
    /// Tries per piece.
    pub tries: u8,
    /// Each piece's size in bytes, in order; its sound was wiped on arrival.
    pub sizes: Vec<usize>,
    /// The pieces whose text was told.
    pub told: usize,
    /// Where it stands.
    pub state: CloudState,
}

impl CloudLog {
    /// `ask`'s record, sending; its key and pieces are dropped here, which wipes them.
    pub(super) fn new(ask: CloudAsk) -> Self {
        let CloudAsk {
            job,
            shape,
            url,
            header,
            key,
            model,
            lang,
            limits,
            wait_s,
            tries,
            pieces,
        } = ask;
        drop(key);
        let sizes = pieces.iter().map(|p| p.expose().len()).collect();
        drop(pieces);
        let (told, state) = (0, CloudState::Sending);
        Self {
            job,
            shape,
            url,
            header,
            model,
            lang,
            limits,
            wait_s,
            tries,
            sizes,
            told,
            state,
        }
    }

    /// The pieces its drop cancelled: those not told when it was dropped while sending.
    #[must_use]
    pub fn cancelled(&self) -> usize {
        if self.state == CloudState::Dropped {
            self.sizes.len() - self.told
        } else {
            0
        }
    }

    /// Tells `answer` as its next piece's note, ending it on a fault.
    pub(super) fn note(&mut self, answer: CloudAnswer) -> CloudNote {
        match answer {
            CloudAnswer::Text(text) => {
                let index = u16::try_from(self.told).unwrap_or(u16::MAX);
                self.told += 1;
                CloudNote::Piece { index, text }
            }
            CloudAnswer::Fault(fault) => self.end(fault),
        }
    }

    /// Ends it on `fault` and gives the note telling it.
    pub(super) fn end(&mut self, fault: CloudFault) -> CloudNote {
        self.state = CloudState::Fault(fault);
        CloudNote::Fault(fault)
    }
}

/// One key test: its ask without the key, and its answer once given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloudTestLog {
    /// The job's limits.
    pub job: JobLimits,
    /// The row's shape.
    pub shape: CloudShape,
    /// The test address.
    pub url: HttpsUrl,
    /// The header that carried the key.
    pub header: KeyHeader,
    /// The longest wait for the answer, in seconds.
    pub wait_s: u16,
    /// Its answer; `None` while it is held.
    pub answer: Option<Result<(), CloudFault>>,
}

impl CloudTestLog {
    /// `ask`'s record, unanswered; its key is dropped here, which wipes it.
    pub(super) fn new(ask: CloudTest) -> Self {
        let CloudTest {
            job,
            shape,
            url,
            header,
            key,
            wait_s,
        } = ask;
        drop(key);
        Self {
            job,
            shape,
            url,
            header,
            wait_s,
            answer: None,
        }
    }
}

/// Every run and key test the fake had, oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CloudCalls {
    /// Each run `send` opened.
    pub runs: Vec<CloudLog>,
    /// Each key test that passed its check.
    pub tests: Vec<CloudTestLog>,
}
