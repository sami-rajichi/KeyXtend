//! The fake cloud speech, held to the port: asks checked by their own rule, scripted texts and every fault told in
//! order, scripts taken in turn, a failed send, and pieces counted by size with no key, sound or text kept or shown.

use super::*;
use kx_module_api::ports::{
    CloudLimits, CloudRun, CloudShape, FormFields, HttpsUrl, JobLimits, KeyHeader,
};
use kx_module_api::{REDACTED_MARKER, Redacted, SecretBytes, Zeroizing};
use std::sync::Mutex;

mod keys;
mod life;
mod props;

/// How long a test waits for another thread; it never sleeps.
const WITHIN: Duration = Duration::from_secs(10);
/// A key no output may show.
const KEY: &str = "gsk_live-7731-zebra";
/// Groq's address, from its row.
const GROQ: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
/// Groq's key test address.
const MODELS: &str = "https://api.groq.com/openai/v1/models";
/// A transcript no output may show.
const SAID: &str = "bonjour-4417";
/// One sample of sound no output may show; a piece repeats it.
const SAMPLE: [u8; 2] = [0x7f, 0x3e];
/// `SAMPLE` as a plain `Debug` prints it.
const SAMPLE_SHOWN: &str = "127, 62";
/// Every fault a run may tell, as `Range` is only `send`'s own refusal.
const FAULTS: [CloudFault; 12] = [
    CloudFault::Key,
    CloudFault::Credit,
    CloudFault::Allowance { renews_in: None },
    CloudFault::Allowance {
        renews_in: Some(Duration::from_secs(3_600)),
    },
    CloudFault::Rate,
    CloudFault::TooBig,
    CloudFault::Language,
    CloudFault::Empty,
    CloudFault::Strange,
    CloudFault::Network,
    CloudFault::Timeout,
    CloudFault::Gone,
];

/// What a test run heard.
type Heard = Arc<Mutex<Vec<CloudNote>>>;

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// A piece of `bytes` bytes of sound, whole samples.
fn sound(bytes: usize) -> SecretBytes {
    Redacted::new(Zeroizing::new(SAMPLE.repeat(bytes / SAMPLE.len())))
}

/// `text` as a scripted answer.
fn text(text: &str) -> CloudAnswer {
    CloudAnswer::text(text)
}

/// The note telling piece `index`'s `text`.
fn piece(index: u16, text: &str) -> CloudNote {
    let text = secret(text);
    CloudNote::Piece { index, text }
}

/// Test values; the real ones come from the voice module's settings.
fn job() -> JobLimits {
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

/// The upload form as Groq names its fields.
fn form() -> CloudShape {
    let fields = FormFields {
        lang: "language".into(),
        list: false,
        fixed: Vec::new(),
        paywall: false,
    };
    CloudShape::Form(fields)
}

/// French pieces of these sizes, in bytes, for Groq's turbo model.
fn ask(sizes: &[usize]) -> CloudAsk {
    let limits = CloudLimits {
        max_mb: 25,
        max_s: 600,
        per_min: 20,
        at_once: 2,
    };
    CloudAsk {
        job: job(),
        shape: form(),
        url: HttpsUrl::new(GROQ).unwrap(),
        header: KeyHeader::Bearer,
        key: secret(KEY),
        model: Some(Redacted::new("whisper-large-v3-turbo".into())),
        lang: "fr".into(),
        limits,
        wait_s: 60,
        tries: 2,
        pieces: sizes.iter().map(|n| sound(*n)).collect(),
    }
}

/// Groq's key test: its models list.
fn key_test() -> CloudTest {
    CloudTest {
        job: job(),
        shape: form(),
        url: HttpsUrl::new(MODELS).unwrap(),
        header: KeyHeader::Bearer,
        key: secret(KEY),
        wait_s: 60,
    }
}

/// A `tell` that records each note into the returned list.
fn hearer() -> (Heard, Tell) {
    let heard = Heard::default();
    let log = Arc::clone(&heard);
    (heard, Box::new(move |note| log.lock().unwrap().push(note)))
}

/// A run sent with pieces of `sizes` whose notes are recorded.
fn sent(fake: &FakeCloud, sizes: &[usize]) -> (Box<dyn CloudRun>, Heard) {
    let (heard, tell) = hearer();
    (fake.send(ask(sizes), tell).unwrap(), heard)
}

/// The notes `heard` holds, in order.
fn notes(heard: &Heard) -> Vec<CloudNote> {
    heard.lock().unwrap().clone()
}

#[test]
fn send_checks_the_ask_by_its_own_rule_and_opens_no_run() {
    let fake = FakeCloud::new();
    let odd = CloudAsk {
        wait_s: 0,
        ..ask(&[2])
    };
    let refused = fake.send(odd, Box::new(|_| {})).err();
    assert_eq!(refused, Some(CloudFault::Range("wait_s")));
    assert!(fake.calls().runs.is_empty(), "a refused ask sends nothing");
}

#[test]
fn scripted_texts_are_told_in_order_then_done() {
    let fake = FakeCloud::new();
    fake.script(vec![text("un"), text("deux"), text("trois")]);
    let (_run, heard) = sent(&fake, &[4, 6, 2]);
    let want = [piece(0, "un"), piece(1, "deux"), piece(2, "trois")];
    assert_eq!(notes(&heard), [&want[..], &[CloudNote::Done]].concat());
    let run = &fake.calls().runs[0];
    assert_eq!((run.told, run.state), (3, CloudState::Done));
}

#[test]
fn every_fault_is_told_as_scripted_and_ends_the_run() {
    for fault in FAULTS {
        let fake = FakeCloud::new();
        fake.script(vec![text(SAID), CloudAnswer::Fault(fault), text(SAID)]);
        let (_run, heard) = sent(&fake, &[2, 2, 2]);
        let want = [piece(0, SAID), CloudNote::Fault(fault)];
        assert_eq!(notes(&heard), want, "{fault:?}");
        assert!(!fake.answer(0, vec![text(SAID)]), "{fault:?} ended it");
        let run = &fake.calls().runs[0];
        assert_eq!((run.told, run.state), (1, CloudState::Fault(fault)));
    }
}

#[test]
fn each_run_takes_its_own_script_in_turn() {
    let fake = FakeCloud::new();
    fake.script(vec![text("a")]);
    fake.script(vec![CloudAnswer::Fault(CloudFault::Rate)]);
    let heard: Vec<_> = (0..3).map(|_| notes(&sent(&fake, &[2]).1)).collect();
    let rate = CloudNote::Fault(CloudFault::Rate);
    let want = [vec![piece(0, "a"), CloudNote::Done], vec![rate], Vec::new()];
    assert_eq!(heard, want, "a run with no script holds its pieces");
}

#[test]
fn a_failed_send_fails_once_and_opens_no_run() {
    let fake = FakeCloud::new();
    fake.fail(CloudFault::Gone);
    fake.script(vec![text("a")]);
    let failed = fake.send(ask(&[2]), Box::new(|_| {})).err();
    assert_eq!(failed, Some(CloudFault::Gone));
    assert!(fake.calls().runs.is_empty());
    let (_run, heard) = sent(&fake, &[2]);
    assert_eq!(notes(&heard), [piece(0, "a"), CloudNote::Done], "it waited");
}

#[test]
fn pieces_are_counted_by_size_and_the_ask_kept_without_its_key_or_sound() {
    let fake = FakeCloud::new();
    let (_run, _heard) = sent(&fake, &[4, 8, 2]);
    let (run, want) = (&fake.calls().runs[0], ask(&[2]));
    assert_eq!(run.sizes, [4, 8, 2]);
    assert_eq!(
        (&run.url, &run.header, &run.shape),
        (&want.url, &want.header, &want.shape)
    );
    assert_eq!((&run.model, &run.lang), (&want.model, &want.lang));
    assert_eq!((run.limits, run.job), (want.limits, want.job));
    assert_eq!((run.wait_s, run.tries), (want.wait_s, want.tries));
    assert_eq!((run.told, run.state), (0, CloudState::Sending));
}

#[test]
fn debug_never_shows_the_key_the_sound_a_text_or_the_address() {
    let fake = FakeCloud::new();
    fake.script(vec![text(SAID)]);
    let (_run, _heard) = sent(&fake, &[2, 2]);
    assert_eq!(fake.test(key_test()), Ok(()));
    let shown = format!("{fake:?} {:?} {:?}", fake.calls(), text(SAID));
    for secret in [KEY, SAID, "api.groq.com", SAMPLE_SHOWN] {
        assert!(!shown.contains(secret), "{shown} shows {secret}");
    }
    assert!(shown.contains(REDACTED_MARKER), "{shown}");
    assert!(
        shown.contains("runs: 1") && shown.contains("tests: 1"),
        "{shown}"
    );
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never tells")]
fn a_range_fault_in_a_script_is_a_test_mistake() {
    let odd = CloudAnswer::Fault(CloudFault::Range("lang"));
    FakeCloud::new().script(vec![odd]);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn a_fault_send_never_returns_is_a_test_mistake() {
    FakeCloud::new().fail(CloudFault::Key);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never returns")]
fn a_failed_range_is_the_asks_own_check_not_a_script() {
    FakeCloud::new().fail(CloudFault::Range("wait_s"));
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "never answers")]
fn a_range_answer_for_a_key_test_is_a_test_mistake() {
    FakeCloud::new().script_test(Err(CloudFault::Range("key")));
}
