//! One behaviour suite for every `CloudSpeech`, and the made-up asks it sends: the fake and the worker's adapter each
//! run it with their own way of playing the service, so both keep the port's promises alike.

use super::CloudAnswer;
use kx_module_api::ports::{
    CloudAsk, CloudFault, CloudLimits, CloudNote, CloudRun, CloudShape, CloudSpeech, CloudTest,
    FormFields, HttpsUrl, JobLimits, KeyHeader, SAMPLE_BYTES, SOUND_HZ,
};
use kx_module_api::{Redacted, Secret, SecretBytes, Zeroizing};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

/// An example address.
pub const ADDRESS: &str = "https://stt.example.com/v1/audio/transcriptions";
/// The example's key test address.
pub const MODELS: &str = "https://stt.example.com/v1/models";
/// A made-up model id no output may show, as a typed one is the user's.
pub const MODEL: &str = "made-up-model-v3";
/// The language the asks send.
pub const LANG: &str = "fr";
/// The wait the asks give, as the `[voice]` default does.
pub const WAIT_S: u16 = 60;
/// The tries the asks give, as the `[voice]` default does.
pub const TRIES: u8 = 2;
/// One sample of sound no output may show; a piece repeats it.
pub const SAMPLE: [u8; SAMPLE_BYTES] = [0x7f, 0x3e];
/// Bytes of a second of sound.
pub const SECOND: usize = SOUND_HZ * SAMPLE_BYTES;
/// How long the suite waits for a note; a wait, never a sleep.
pub const WITHIN: Duration = Duration::from_secs(10);

/// The callback a run tells its notes to.
pub type Tell = Box<dyn Fn(CloudNote) + Send>;

/// A cloud speech under the suite, with its own way of playing the service.
pub trait Served {
    /// The service itself, for asks it refuses before anything is played.
    fn speech(&self) -> &dyn CloudSpeech;

    /// Sends `ask`; the service answers its pieces with `answers` in order, then ends a run whose pieces all have text.
    ///
    /// # Errors
    /// The send's own refusal.
    fn run(
        &self,
        ask: CloudAsk,
        answers: Vec<CloudAnswer>,
        tell: Tell,
    ) -> Result<Box<dyn CloudRun>, CloudFault>;

    /// Tests `ask`'s key, which the service answers with `answer`.
    ///
    /// # Errors
    /// The test's fault.
    fn test(&self, ask: CloudTest, answer: Result<(), CloudFault>) -> Result<(), CloudFault>;
}

/// Runs the whole suite on `served`, failing the test that calls it at the first broken promise.
pub fn check(served: &dyn Served) {
    texts_come_back_in_order_then_the_end(served);
    a_fault_is_told_once_and_ends_the_run(served);
    a_bad_ask_or_test_is_refused_by_its_value(served);
    a_key_test_answers_as_the_service_does(served);
}

/// A made-up key no output may show, built as the test runs so no key-like text sits in the source.
#[must_use]
pub fn key() -> String {
    ["kx", "made", "up", "key"].join("-")
}

/// `text` as a secret.
#[must_use]
pub fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// `bytes` of whole samples as secret sound.
#[must_use]
pub fn sound(bytes: usize) -> SecretBytes {
    Redacted::new(Zeroizing::new(SAMPLE.repeat(bytes / SAMPLE_BYTES)))
}

/// The job's limits as the `[voice]` defaults give them.
#[must_use]
pub const fn job() -> JobLimits {
    JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

/// A plain upload form.
#[must_use]
pub fn form() -> CloudShape {
    CloudShape::Form(FormFields {
        lang: "language".into(),
        list: false,
        fixed: Vec::new(),
        paywall: false,
        region: None,
    })
}

/// Made-up limits, as a row might set them: at most ten minutes a request.
#[must_use]
pub const fn limits() -> CloudLimits {
    CloudLimits {
        max_mb: 25,
        max_s: 600,
        per_min: 20,
        at_once: 2,
    }
}

/// A French run of pieces of these sizes, in bytes, for the made-up model at the example address.
#[must_use]
pub fn ask(sizes: &[usize]) -> CloudAsk {
    CloudAsk {
        job: job(),
        shape: form(),
        url: example(ADDRESS),
        header: KeyHeader::Bearer,
        key: secret(&key()),
        model: Some(Redacted::new(MODEL.to_owned())),
        lang: LANG.to_owned(),
        limits: limits(),
        wait_s: WAIT_S,
        tries: TRIES,
        pieces: sizes.iter().map(|&n| sound(n)).collect(),
    }
}

/// A key test: the example's models list.
#[must_use]
pub fn key_test() -> CloudTest {
    CloudTest {
        job: job(),
        shape: form(),
        url: example(MODELS),
        header: KeyHeader::Bearer,
        key: secret(&key()),
        wait_s: WAIT_S,
    }
}

/// `text`, one of the example addresses, as an address.
#[allow(clippy::expect_used, reason = "the example addresses are HTTPS")]
fn example(text: &str) -> HttpsUrl {
    HttpsUrl::new(text).expect("an HTTPS address")
}

/// A `tell` and what it hears.
fn hearer() -> (Tell, Receiver<CloudNote>) {
    let (said, heard) = mpsc::channel();
    (Box::new(move |note| drop(said.send(note))), heard)
}

/// The next `n` notes `heard` is told, each within `WITHIN`; fewer when one is late.
fn next(heard: &Receiver<CloudNote>, n: usize) -> Vec<CloudNote> {
    (0..n)
        .map_while(|_| heard.recv_timeout(WITHIN).ok())
        .collect()
}

/// The port's text for piece `index`.
fn piece(index: u16, text: &str) -> CloudNote {
    CloudNote::Piece {
        index,
        text: secret(text),
    }
}

/// Each piece's text comes back in order, an empty one too, then the end.
fn texts_come_back_in_order_then_the_end(served: &dyn Served) {
    let (tell, heard) = hearer();
    let answers = vec![CloudAnswer::text("one"), CloudAnswer::text("")];
    let run = served.run(ask(&[SECOND, SECOND]), answers, tell);
    assert!(run.is_ok(), "the run starts");
    let want = [piece(0, "one"), piece(1, ""), CloudNote::Done];
    assert_eq!(next(&heard, want.len()), want);
}

/// A fault after a text is told once, and nothing follows it.
fn a_fault_is_told_once_and_ends_the_run(served: &dyn Served) {
    let (tell, heard) = hearer();
    let fault = CloudFault::Network;
    let answers = vec![CloudAnswer::text("one"), CloudAnswer::Fault(fault)];
    let run = served.run(ask(&[SECOND, SECOND]), answers, tell);
    assert!(run.is_ok(), "the run starts");
    let want = [piece(0, "one"), CloudNote::Fault(fault)];
    assert_eq!(next(&heard, want.len()), want);
    assert!(heard.try_recv().is_err(), "nothing after the fault");
}

/// An ask with no piece and a test with no key are refused by the value's name before anything is played.
fn a_bad_ask_or_test_is_refused_by_its_value(served: &dyn Served) {
    let (tell, _heard) = hearer();
    let empty = CloudAsk {
        pieces: Vec::new(),
        ..ask(&[SECOND])
    };
    let refused = served.speech().send(empty, tell).err();
    assert_eq!(refused, Some(CloudFault::Range("pieces")));
    let keyless = CloudTest {
        key: secret(""),
        ..key_test()
    };
    let refused = served.speech().test(keyless);
    assert_eq!(refused, Err(CloudFault::Range("key")));
}

/// A key test answers what the service says: the key works, or it is refused.
fn a_key_test_answers_as_the_service_does(served: &dyn Served) {
    assert_eq!(served.test(key_test(), Ok(())), Ok(()));
    let refused = Err(CloudFault::Key);
    assert_eq!(served.test(key_test(), refused), refused);
}
