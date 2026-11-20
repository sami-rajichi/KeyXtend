//! `AssemblyAI`'s Sync request [A1][A2]: one POST with the key only in `Authorization`, the model only in the row's own
//! header, and a body of the JSON `config` part first, holding one language code and nothing else, then the piece as
//! the WAV `audio` part; a run with no model or code, or another shape's call, is refused, no key test is ever sent,
//! and a piece is posted and read end to end. `replies` reads each answer as its pages say.

mod replies;

use super::form::{field, names, parts, piece};
use super::*;
use crate::net::assembly::Assembly;
use crate::net::send::{self, Sending};
use crate::net::shape::{self, Request, Shape};
use kx_ipc::{NetFault, ShapeCode};
use kx_platform_net::Verb;
use proptest::prelude::*;
use std::sync::atomic::AtomicBool;

/// The one address the tests send to, never the table's.
const ADDRESS: &str = "https://stt.example.com/v1/transcribe";
/// A made-up header for the model, so a name held in the worker would fail.
const MODEL_HEADER: &str = "X-Test-Model";
/// A made-up model, as the row offers one.
const UNIVERSAL: &str = "made-up-model-pro";
/// The sound the tests send.
const WAV: &[u8] = b"RIFF-and-samples";
/// What the text fixture says.
const SAID: &str = "Bonjour tout le monde.";
/// The header the key goes in, as the row's `bearer` names it.
const AUTHORIZATION: &str = "Authorization";

/// A call to `url` in `AssemblyAI`'s shape, the key as a bearer.
fn assembly(url: &'static str) -> NetCall<'static> {
    let model_header = Cow::Borrowed(MODEL_HEADER);
    call_to(url, NetShape::Assembly { model_header })
}

/// The request for a piece of `WAV` in `lang`.
fn built(lang: &str) -> Request {
    let call = assembly(ADDRESS);
    Assembly
        .build(&piece(&call, Some(UNIVERSAL), lang, WAV))
        .unwrap()
}

#[test]
fn one_post_sends_the_config_part_first_then_the_wav_audio_part_and_nothing_else() {
    let req = built("fr");
    assert_eq!(req.verb, Verb::Post);
    assert_eq!(names(&req), ["config", "audio"], "config first [A1]");
    let got = parts(&req);
    let config = br#"{"language_codes":["fr"]}"#;
    assert_eq!(got[0].3, config, "one code, no timestamps, prompt or terms");
    assert_eq!(got[0].1, None, "a text part");
    assert!(
        got[0].2.ends_with("\r\nContent-Type: application/json"),
        "{}",
        got[0].2
    );
    assert_eq!(
        (got[1].1.as_deref(), got[1].3.as_slice()),
        (Some("piece.wav"), WAV)
    );
    assert!(
        got[1].2.ends_with("\r\nContent-Type: audio/wav"),
        "{}",
        got[1].2
    );
}

#[test]
fn the_model_goes_only_in_its_own_header_and_the_key_only_in_authorization() {
    let req = built("en");
    let heads: Vec<&str> = req.fields.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(heads, [AUTHORIZATION, MODEL_HEADER, "Content-Type"]);
    assert_eq!(field(&req, MODEL_HEADER), Some(UNIVERSAL.to_owned()));
    let bearer = format!("Bearer {}", made_key());
    assert_eq!(field(&req, AUTHORIZATION), Some(bearer));
    let holds = |hay: &[u8], pat: &[u8]| hay.windows(pat.len()).any(|w| w == pat);
    assert!(
        !holds(&req.body, made_key().as_bytes()),
        "the body holds no key"
    );
    assert!(!holds(&req.body, UNIVERSAL.as_bytes()), "nor the model");
    let others = req.fields.iter().filter(|(n, _)| n != AUTHORIZATION);
    assert!(
        others
            .clone()
            .all(|(_, v)| !holds(v, made_key().as_bytes()))
    );
}

#[test]
fn a_run_with_no_model_or_code_another_shapes_call_or_a_breaking_model_is_out_of_range() {
    let call = assembly(ADDRESS);
    let refused = |model, lang| Assembly.build(&piece(&call, model, lang, WAV)).err();
    assert_eq!(
        refused(None, "fr"),
        Some(NetFault::Range),
        "a model on every request [A1]"
    );
    assert_eq!(
        refused(Some(UNIVERSAL), ""),
        Some(NetFault::Range),
        "one code, never none"
    );
    for bad in ["a\r\nb", "a\nb", "a\0b", "a b"] {
        assert_eq!(refused(Some(bad), "fr"), Some(NetFault::Range), "{bad:?}");
    }
    let form = call_to(ADDRESS, NetShape::Form(form()));
    let other = Assembly
        .build(&piece(&form, Some(UNIVERSAL), "fr", WAV))
        .err();
    assert_eq!(other, Some(NetFault::Range));
}

#[test]
fn a_model_header_naming_a_header_the_request_sets_is_out_of_range() {
    let key = NetHeader::Named(Cow::Borrowed("X-Test-Key"));
    let bearer = [
        "authorization",
        "CONTENT-TYPE",
        "Content-Length",
        "host",
        "USER-agent",
    ];
    let cases = bearer.map(|n| (n, NetHeader::Bearer));
    for (name, header) in cases.into_iter().chain([("x-test-key", key)]) {
        let model_header = Cow::Borrowed(name);
        let call = NetCall {
            header,
            ..call_to(ADDRESS, NetShape::Assembly { model_header })
        };
        let got = Assembly
            .build(&piece(&call, Some(UNIVERSAL), "en", WAV))
            .err();
        assert_eq!(got, Some(NetFault::Range), "{name}");
    }
}

#[test]
fn it_has_no_key_test_so_no_request_goes_without_sound() {
    assert_eq!(
        Assembly.test(&assembly(ADDRESS)).err(),
        Some(NetFault::Range)
    );
    let sending = sending(assembly(ADDRESS), "");
    let http = Arc::new(FakeHttp::new([]));
    let clock = Arc::new(Still::default());
    let got = send::test(&kit(&http, &clock), &sending);
    assert_eq!(got, Err(NetFault::Range));
    assert!(http.called().is_empty(), "nothing is sent [A1]");
}

#[test]
fn it_keeps_the_header_a_429_or_a_503_waits_by() {
    assert!(built("fr").keep.contains(&"retry-after"));
}

/// What a run of `call` in `lang` sends with, once.
fn sending(call: NetCall<'static>, lang: &'static str) -> Sending {
    Sending {
        shape: shape::of(ShapeCode::Assembly),
        call,
        model: Some(Cow::Borrowed(UNIVERSAL)),
        lang: Cow::Borrowed(lang),
        tries: 1,
        pace: None,
        dropped: Arc::new(AtomicBool::new(false)),
    }
}

#[test]
fn the_job_builds_it_and_a_piece_posts_its_parts_and_reads_its_text() {
    let http = Arc::new(FakeHttp::new([reply(OK, replies::TEXT)]));
    let clock = Arc::new(Still::default());
    let got = send::piece(&kit(&http, &clock), &sending(assembly(ADDRESS), "fr"), WAV);
    assert_eq!(got.map(|t| t.to_string()), Ok(SAID.to_owned()));
    let called = http.called();
    let [one] = &called[..] else {
        panic!("one call, not {}", called.len())
    };
    assert_eq!((one.url.as_str(), one.verb), (ADDRESS, Verb::Post));
    let model = one.fields.iter().find(|(n, _)| n == MODEL_HEADER);
    assert_eq!(model.map(|(_, v)| v.as_slice()), Some(UNIVERSAL.as_bytes()));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn any_code_reads_back_whole_from_the_config_part(lang in "\\PC{1,16}") {
        let req = built(&lang);
        let config: serde_json::Value = serde_json::from_slice(&parts(&req)[0].3).unwrap();
        prop_assert_eq!(config, serde_json::json!({ "language_codes": [lang] }));
    }
}
