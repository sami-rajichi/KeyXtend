//! A cloud dictation through the actor (ADR-0047): the listen-only run hands its speech over at Stop, the key is read
//! just before `send`, and the service's texts are typed, kept and counted under the pick; the run is dropped once it
//! ends, and Cancel drops its requests. Keys and hidden runs are in `keys`, the key page's effects in `page`, the local
//! model's take in `local`, the missing port and the row's piece length in `port`, the card's warning in `warn`, Open
//! service in `settle`, the stop flag's window in `window`, a used allowance's renewal in `renewal`, the run's table in
//! `table`, and the Models page's side in `side`.

mod keys;
mod local;
mod page;
mod port;
mod remove;
mod renewal;
mod settle;
mod side;
mod table;
mod trial;
mod typed;
mod warn;
mod window;

use std::sync::Arc;

use kx_module_api::ports::{
    BarButton, BarHint, BarMark, BarNote, CloudFault, CloudWhy, LocalUse, MAX_BLOB_BYTES,
    ServiceName, SpeechNote, VoiceBar, VoiceClick,
};
use kx_module_api::{Redacted, Secret, Zeroizing};
use kx_platform_fake::{CloudAnswer, CloudState};
use kx_test_support::host::FakeHost;

use super::rig::{Fakes, Rig, done};
use crate::counts::{Device, Model};
use crate::keys::Keys;
use crate::services::piece_s;
use crate::services::tests::shipped::{entry, first, model, pick};
use crate::store::Store;

/// The pick English dictates with: the first offered row's first model.
fn pick0() -> String {
    pick(first(), 0)
}
/// A tenth of a second of sound, as one piece.
const PIECE: usize = 3_200;
/// The test's own key for the fake service, built as the test runs so no key-like text sits in the source.
pub(super) fn key() -> String {
    ["kx", "fake", "key"].join("-")
}
/// The longest piece the first row takes, held to the most a run cuts.
fn piece_s0() -> u8 {
    piece_s(&first().limits)
}

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// `text` as a settings table.
fn table(text: &str) -> toml::Value {
    toml::Value::Table(text.parse().unwrap())
}

/// Sets `key` as Groq's, the first offered row.
pub(super) fn set_key(fake: &Fakes, key: &str) {
    let keys = Keys::new(Arc::new(fake.shelf.clone()), false);
    keys.set(&first().id, secret(key)).unwrap();
}

/// Groq picked for English, switched on with its terms confirmed, and its key set.
pub(super) fn groq(fake: &Fakes, host: &mut FakeHost) {
    host.put("cloud", true.into());
    host.put("services", table(&entry(first(), "")));
    host.put("picks", table(&format!("en = \"{}\"", pick0())));
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
        match self.views().last()?.mark.clone()? {
            BarMark::Cloud(name) => Some(name),
            BarMark::Device(_) => None,
        }
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
        (None, piece_s0()),
        "listen-only, cut by the row"
    );
    assert_eq!(rig.mark(), Some(first().shown()));
    assert!(
        rig.fake.cloud.calls().runs.is_empty(),
        "nothing leaves while it listens"
    );
    rig.key();
    assert!(
        rig.fake.cloud.calls().runs.is_empty(),
        "nothing leaves before the hand-over"
    );
    rig.tell(SpeechNote::Done(done(0)));
    let runs = rig.fake.cloud.calls().runs;
    assert_eq!(runs.len(), 1);
    let sent = runs[0].model.as_ref().map(|m| m.expose().clone());
    let listed = model(first(), 0).id.as_str();
    assert_eq!(
        (runs[0].lang.as_str(), sent.as_deref()),
        ("en", Some(listed))
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
    assert_eq!(*opened.history.entries()[0].model.expose(), pick0());
    let bucket = opened.counts.rows().keys().next().unwrap().clone();
    let model = Model::Cloud {
        service: first().id.clone(),
        model: Some(model(first(), 0).id.clone()),
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
