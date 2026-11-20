use super::*;
use crate::played::stage::{INDEX, PIECE, SEND, SOUND_MS, STAGE, STAGE_LINE};
use crate::played::{cloud, logged};
use kx_mod_voice::VoiceSettings;
use kx_mod_voice::cloud::Chosen;
use kx_mod_voice::services::{Model, Service};
use kx_module_api::ports::CloudTest;
use kx_module_api::{Redacted, Zeroizing};
use kx_platform_fake::{CloudAnswer, FakeCloud};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// The rows the tests send to: one at its own address, and Custom at a typed one.
const GROQ: &str = "groq";
const CUSTOM: &str = "custom";
/// A row kept in the table but not offered, whose address Custom may still be pointed at.
const HIDDEN: &str = "openai";
/// A typed Custom address on a test host of its own, and a typed model id.
const TYPED: &str = "https://stt.example.test/v1/audio/transcriptions";
const TYPED_MODEL: &str = "whisper-test";
/// The test's own key, made here, and what the service answers; no line may hold either.
const KEY: &str = "test-key-made-here";
const SAID: &str = "Sfax by the sea";
/// The piece's length, and the level the stage lines take.
const PIECE_MS: u32 = 1000;
const DEBUG: Option<&str> = Some("debug");

/// The row `id` names.
fn row(id: &str) -> &'static Service {
    table().unwrap().get(id).unwrap()
}

/// The test's key as the port takes it.
fn key() -> kx_module_api::Secret {
    Redacted::new(Zeroizing::new(KEY.to_owned()))
}

/// An English ask from `row` at its own address or at `typed`, with one piece of `PIECE_MS`, as the actor builds one
/// for its model `listed`, or for the typed model when none, with the keep-nothing switch at `keep`.
fn ask_as(row: &Service, typed: &str, listed: Option<&Model>, keep: bool) -> CloudAsk {
    let s = VoiceSettings::from_defaults().unwrap();
    let model = listed
        .map_or(Chosen::Typed(TYPED_MODEL), Chosen::Listed)
        .sent();
    let lang = listed.and_then(|m| m.code("en")).unwrap_or("en");
    let per_s = SOUND_HZ * SAMPLE_BYTES;
    let bytes = per_s * usize::try_from(PIECE_MS / MS_PER_S).unwrap();
    CloudAsk {
        job: s.job(),
        shape: row.shape_for(listed, keep),
        url: row.url(typed).unwrap(),
        header: row.header.clone(),
        key: key(),
        model: model.map(|m| Redacted::new(m.to_owned())),
        lang: lang.to_owned(),
        limits: row.limits,
        wait_s: s.net_wait_s,
        tries: s.net_tries,
        pieces: vec![Redacted::new(Zeroizing::new(vec![0; bytes]))],
    }
}

/// An ask from `row` with its first model, or the typed one, at its own address or at `typed`.
fn ask(row: &Service, typed: &str) -> CloudAsk {
    ask_as(row, typed, row.models.first(), false)
}

/// Each row's address: its own, or `TYPED` for a typed row.
fn address(row: &Service) -> &'static str {
    if matches!(row.address, Address::Typed { .. }) {
        TYPED
    } else {
        ""
    }
}

#[test]
fn a_written_cloud_dictation_names_its_service_and_never_its_text_key_or_address() {
    let (fake, heard) = (FakeCloud::new(), Arc::new(AtomicUsize::new(0)));
    fake.script(vec![CloudAnswer::text(SAID)]);
    let count = Arc::clone(&heard);
    let tell = Box::new(move |_| {
        count.fetch_add(1, Ordering::SeqCst);
    });
    let sent = cloud(Arc::new(fake));
    let groq = row(GROQ);
    let text = logged::at(None, || drop(sent.send(ask(groq, ""), tell).unwrap()));
    assert_eq!(logged::values(&text, WROTE_LINE, SERVICE), [GROQ], "{text}");
    assert!(!text.contains(FAILED_LINE), "{text}");
    let host = groq.url("").unwrap().host().to_owned();
    for secret in [SAID, KEY, host.as_str()] {
        assert!(!text.contains(secret), "{secret} leaked: {text}");
    }
    assert_eq!(
        heard.load(Ordering::SeqCst),
        2,
        "the piece and the end go on"
    );
}

#[test]
fn a_failed_cloud_dictation_names_its_service_and_its_faults_kind_alone() {
    let fake = FakeCloud::new();
    let sent = Sent(Arc::new(fake.clone()));
    let groq = row(GROQ);
    let renews_in = Some(Duration::from_secs(3600));
    let text = logged::at(None, || {
        fake.fail(CloudFault::Gone);
        assert!(sent.send(ask(groq, ""), Box::new(|_| {})).is_err());
        fake.script(vec![CloudAnswer::Fault(CloudFault::Allowance {
            renews_in,
        })]);
        drop(sent.send(ask(groq, ""), Box::new(|_| {})).unwrap());
    });
    assert_eq!(
        logged::values(&text, FAILED_LINE, FAULT),
        ["Gone", "Allowance"]
    );
    assert_eq!(logged::values(&text, FAILED_LINE, SERVICE), [GROQ, GROQ]);
    assert!(!text.contains(WROTE_LINE), "nothing was written: {text}");
    assert!(!text.contains("3600"), "never a value it holds: {text}");
}

#[test]
fn each_rows_own_ask_names_that_row_with_any_of_its_models_and_either_switch() {
    for r in table().unwrap().rows() {
        let listed = r.models.iter().map(Some);
        let other = (r.other || r.models.is_empty()).then_some(None);
        let models: Vec<Option<&Model>> = listed.chain(other).collect();
        for (model, keep) in models.into_iter().flat_map(|m| [(m, false), (m, true)]) {
            let a = ask_as(r, address(r), model, keep);
            assert_eq!(service_of(&a), Some(r.id.as_str()), "{} {keep}", r.id);
        }
    }
}

#[test]
fn groqs_own_ask_is_groq_and_custom_at_groqs_address_is_custom() {
    let (groq, custom) = (row(GROQ), row(CUSTOM));
    let at = groq.url("").unwrap();
    for model in groq.models.iter().map(Some).chain([None]) {
        assert_eq!(service_of(&ask_as(groq, "", model, false)), Some(GROQ));
    }
    let mut a = ask_as(custom, at.expose(), None, false);
    assert_eq!(
        service_of(&a),
        Some(CUSTOM),
        "Custom's own model at Groq's address"
    );
    a.model = groq.models.first().map(|m| Redacted::new(m.id.clone()));
    assert_eq!(service_of(&a), Some(CUSTOM), "even with Groq's model");
}

#[test]
fn custom_at_a_listed_rows_address_is_named_custom_even_a_hidden_rows() {
    assert!(!row(HIDDEN).offered, "{HIDDEN} is a hidden row");
    let custom = row(CUSTOM);
    let fixed = table().unwrap().rows().iter();
    for r in fixed.filter(|r| matches!(r.address, Address::Fixed(_))) {
        let mut a = ask_as(custom, r.url("").unwrap().expose(), None, false);
        assert_eq!(service_of(&a), Some(CUSTOM), "at {}'s address", r.id);
        a.model = r.models.first().map(|m| Redacted::new(m.id.clone()));
        assert_eq!(service_of(&a), Some(CUSTOM), "with {}'s model", r.id);
    }
}

#[test]
fn the_send_and_each_piece_log_a_debug_stage_with_the_pieces_length() {
    let fake = FakeCloud::new();
    fake.script(vec![CloudAnswer::text(SAID)]);
    let sent = Sent(Arc::new(fake));
    let custom = row(CUSTOM);
    let text = logged::at(DEBUG, || {
        drop(sent.send(ask(custom, TYPED), Box::new(|_| {})).unwrap());
    });
    let stages = logged::values(&text, STAGE_LINE, STAGE);
    assert_eq!(stages, [SEND, PIECE], "{text}");
    assert_eq!(logged::values(&text, STAGE_LINE, INDEX), ["0"]);
    let length = logged::values(&text, STAGE_LINE, SOUND_MS);
    assert_eq!(length, [PIECE_MS.to_string()]);
    assert_eq!(logged::values(&text, WROTE_LINE, SERVICE), [CUSTOM]);
    assert!(!text.contains(TYPED_MODEL), "never a typed id: {text}");
}

#[test]
fn a_key_test_goes_through_as_it_is() {
    let fake = FakeCloud::new();
    fake.script_test(Err(CloudFault::Key));
    let (sent, groq) = (Sent(Arc::new(fake)), row(GROQ));
    let s = VoiceSettings::from_defaults().unwrap();
    let test = CloudTest {
        job: s.job(),
        shape: groq.shape.clone(),
        url: groq.test_url("").unwrap(),
        header: groq.header.clone(),
        key: key(),
        wait_s: s.net_wait_s,
    };
    assert_eq!(sent.test(test), Err(CloudFault::Key));
}
