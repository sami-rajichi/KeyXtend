//! A cloud dictation through the actor (ADR-0047): the listen-only run hands its speech over at Stop, the key is read
//! just before `send`, and the service's texts are typed, kept and counted under the pick; the run is dropped once it
//! ends, and Cancel drops its requests. Keys and hidden runs are in `keys`, the key page's effects in `page`, the local
//! model's take in `local`, and the missing port and the row's piece length in `port`.

mod keys;
mod local;
mod page;
mod port;

use std::sync::Arc;

use kx_module_api::ports::{
    BarButton, BarHint, BarNote, CloudFault, CloudWhy, LocalUse, MAX_BLOB_BYTES, ServiceName,
    SpeechNote, VoiceBar, VoiceClick,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use kx_platform_fake::{CloudAnswer, CloudState};
use kx_test_support::host::FakeHost;

use super::rig::{Fakes, Rig, done};
use crate::counts::{Device, Model};
use crate::keys::Keys;
use crate::store::Store;

/// The pick English dictates with.
const PICK: &str = "groq:whisper-large-v3-turbo";
/// Groq's model, as the request names it.
const MODEL: &str = "whisper-large-v3-turbo";
/// A tenth of a second of sound, as one piece.
const PIECE: usize = 3_200;
/// The test's own key for the fake service, built as the test runs so no key-like text sits in the source.
pub(super) fn key() -> String {
    ["kx", "fake", "key"].join("-")
}
/// The longest piece Groq's row takes, held to the most a run cuts.
const PIECE_S: u8 = 120;

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// `text` as a settings table.
fn table(text: &str) -> toml::Value {
    toml::Value::Table(text.parse().unwrap())
}

/// Sets `key` as Groq's.
pub(super) fn set_key(fake: &Fakes, key: &str) {
    let keys = Keys::new(Arc::new(fake.shelf.clone()), false);
    keys.set("groq", secret(key)).unwrap();
}

/// Groq picked for English, switched on with its terms confirmed, and its key set.
pub(super) fn groq(fake: &Fakes, host: &mut FakeHost) {
    host.put("cloud", true.into());
    host.put("services", table("groq = { on = 2026-11-17, terms = 1 }"));
    host.put("picks", table(&format!("en = \"{PICK}\"")));
    set_key(fake, &key());
}

/// The cloud note for `why`, with "Use the local model" ready and Try again.
fn cloud_note(why: CloudWhy) -> VoiceBar {
    VoiceBar::Note(BarNote {
        hint: BarHint::Cloud {
            why,
            local: LocalUse::Ready,
        },
        act: Some(BarButton::Again),
        mic: false,
    })
}

impl Rig {
    /// Listens with two pieces of speech ready to hand over.
    fn talk(&mut self) {
        self.fake.speech.set_speech(vec![vec![0; PIECE]; 2]);
        self.listen();
    }

    /// Listens, stops, and the run hands its speech over.
    pub(super) fn dictate(&mut self) {
        self.talk();
        self.key();
        self.tell(SpeechNote::Done(done(0)));
    }

    /// The text of every ask typed, in order, without the suffix.
    pub(super) fn typed_texts(&self) -> Vec<String> {
        let suffix = self.actor.settings().type_suffix.clone();
        let text = |t: &str| t.strip_suffix(suffix.as_str()).unwrap_or(t).to_owned();
        self.asks().iter().map(|a| text(a.text.expose())).collect()
    }

    /// The cloud mark of the last view.
    fn mark(&self) -> Option<ServiceName> {
        self.views().last().and_then(|v| v.cloud.clone())
    }
}

#[test]
fn nothing_leaves_before_stop_then_the_speech_goes_and_its_text_is_typed() {
    let mut rig = Rig::build(groq);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.talk();
    let ask = rig.fake.speech.runs()[0].ask.clone();
    assert_eq!(
        (ask.model, ask.piece_s),
        (None, PIECE_S),
        "listen-only, cut by the row"
    );
    assert_eq!(rig.mark(), Some(ServiceName::Named("Groq".into())));
    rig.key();
    assert!(
        rig.fake.cloud.calls().runs.is_empty(),
        "nothing leaves before the hand-over"
    );
    rig.tell(SpeechNote::Done(done(0)));
    let runs = rig.fake.cloud.calls().runs;
    assert_eq!(runs.len(), 1);
    let model = runs[0].model.as_ref().map(|m| m.expose().clone());
    assert_eq!(
        (runs[0].lang.as_str(), model.as_deref()),
        ("en", Some(MODEL))
    );
    assert_eq!(
        (&runs[0].sizes[..], runs[0].state),
        (&[PIECE, PIECE][..], CloudState::Done)
    );
    assert_eq!(rig.typed_texts(), ["hello world"]);
    assert_eq!(rig.mark(), None);
}

#[test]
fn notes_told_before_send_returns_are_heard_and_the_ended_run_is_dropped() {
    let mut rig = Rig::build(groq);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.dictate();
    assert_eq!(rig.typed_texts(), ["hello world"]);
    assert!(rig.actor.net.idle(), "the run that ended was dropped");
}

#[test]
fn the_text_is_kept_and_counted_under_the_service() {
    let mut rig = Rig::build(groq);
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("world")]);
    rig.dictate();
    let opened = Store::open(Arc::new(rig.fake.shelf.clone()), MAX_BLOB_BYTES).unwrap();
    assert_eq!(opened.history.entries()[0].model.expose(), PICK);
    let bucket = opened.counts.rows().keys().next().unwrap().clone();
    let model = Model::Cloud {
        service: "groq".into(),
        model: Some(MODEL.into()),
    };
    assert_eq!((bucket.model, bucket.device), (model, Device::Cloud));
}

#[test]
fn no_speech_handed_over_sends_nothing() {
    let mut rig = Rig::build(groq);
    rig.listen();
    rig.key();
    rig.tell(SpeechNote::Done(done(0)));
    assert!(rig.fake.cloud.calls().runs.is_empty());
    let VoiceBar::Note(note) = rig.bar() else {
        panic!("a note shows");
    };
    assert_eq!(note.hint, BarHint::Nothing);
    assert!(rig.actor.net.idle());
}

#[test]
fn cancel_while_sending_drops_the_requests_and_transcribe_anyway_sends_again() {
    let mut rig = Rig::build(groq);
    rig.fake.cloud.script(vec![CloudAnswer::text("hello")]);
    rig.dictate();
    assert_eq!(rig.fake.cloud.calls().runs[0].state, CloudState::Sending);
    rig.act(VoiceClick::Bar(BarButton::Cancel));
    let run = rig.fake.cloud.calls().runs[0].clone();
    assert_eq!((run.state, run.cancelled()), (CloudState::Dropped, 1));
    assert!(rig.asks().is_empty() && rig.actor.net.idle());
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("hello"), CloudAnswer::text("again")]);
    rig.act(VoiceClick::Bar(BarButton::Anyway));
    assert_eq!(rig.fake.cloud.calls().runs.len(), 2);
    assert_eq!(rig.typed_texts(), ["hello again"]);
}

#[test]
fn a_service_fault_keeps_the_recording_and_try_again_types_once() {
    let mut rig = Rig::build(groq);
    rig.fake.cloud.script(vec![
        CloudAnswer::text("half"),
        CloudAnswer::Fault(CloudFault::Rate),
    ]);
    rig.dictate();
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Rate));
    assert!(rig.asks().is_empty(), "nothing typed");
    rig.fake
        .cloud
        .script(vec![CloudAnswer::text("whole"), CloudAnswer::text("text")]);
    rig.act(VoiceClick::Bar(BarButton::Again));
    assert_eq!(rig.typed_texts(), ["whole text"]);
    assert_eq!(
        rig.fake.speech.runs()[0].pieces,
        2,
        "the same speech went twice"
    );
}

#[test]
fn a_service_that_cannot_start_shows_the_fault_note() {
    let mut rig = Rig::build(groq);
    rig.fake.cloud.fail(CloudFault::Gone);
    rig.dictate();
    assert_eq!(rig.bar(), cloud_note(CloudWhy::Fault));
    assert!(rig.actor.net.idle());
}
