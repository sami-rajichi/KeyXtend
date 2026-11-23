//! Gemini's single request [N1][N6][N10]: one JSON POST with the model, the piece inline as Base64, one language code
//! and `store` off, nothing else, the key only in the call's own header; the body is the WAV a third larger in a small
//! wrapper; a run with no model or language, or another shape's call, is refused; the key's test is a GET with the key
//! alone [N8]; and a piece goes and its text comes back end to end. `replies` reads each answer [N5][N7][N10].

mod replies;

use super::base64::decoded;
use super::bench::{faked, fault, part, send};
use super::form::{field, piece};
use super::*;
use crate::net::gemini::Gemini;
use crate::net::send::{self, Sending};
use crate::net::shape::{self, Request, Shape};
use kx_ipc::{MAX_TAG_BYTES, NetFault, ShapeCode};
use kx_platform_net::Verb;
use serde_json::{Value, json};
use std::sync::atomic::AtomicBool;

/// The one address the tests send to, never the table's.
const ADDRESS: &str = "https://stt.example.com/v1beta/interactions";
/// The one key test address.
const TEST: &str = "https://stt.example.com/v1beta/models";
/// A made-up header for the key, so a name held in the worker would fail.
const HEADER: &str = "X-Test-Key";
/// A made-up model, as the run names one: under `models/`.
const TRANSCRIBE: &str = "models/made-up-3.5-model";
/// Arabic's BCP-47 code, as its set gives it.
const MSA: &str = "ar-SA";
/// The sound the tests send.
const WAV: &[u8] = b"RIFF-and-samples";
/// The JSON's own bytes besides the model, the language and the sound: its names and marks, far within the room the
/// table keeps under a service's cap, whose room test counts one form part's head for them.
const WRAP: usize = 156;

/// A call to `url` in Gemini's shape, the key under its own header.
pub(super) fn gemini(url: &'static str) -> NetCall<'static> {
    NetCall {
        header: NetHeader::Named(Cow::Borrowed(HEADER)),
        ..call_to(url, NetShape::Gemini)
    }
}

/// The request for `wav` with `model` in `lang`.
fn built(model: Option<&str>, lang: &str, wav: &[u8]) -> Result<Request, NetFault> {
    let call = gemini(ADDRESS);
    Gemini.build(&piece(&call, model, lang, wav))
}

/// `req`'s body as JSON, its sound read back from Base64 and left out.
fn json_of(req: &Request) -> (Value, Option<Vec<u8>>) {
    let mut body: Value = serde_json::from_slice(&req.body).unwrap();
    let data = body["input"][0]["data"].take();
    (body, data.as_str().and_then(decoded))
}

#[test]
fn one_post_sends_the_model_the_piece_inline_one_language_and_store_off_and_nothing_else() {
    let req = built(Some(TRANSCRIBE), MSA, WAV).unwrap();
    assert_eq!(req.verb, Verb::Post);
    let (body, sound) = json_of(&req);
    assert_eq!(sound.as_deref(), Some(WAV), "the piece as Base64 [N6]");
    let want = json!({
        "model": TRANSCRIBE,
        "input": [{ "type": "audio", "data": null, "mime_type": "audio/wav" }],
        "generation_config": { "transcription_config": { "language_codes": [MSA] } },
        "store": false,
    });
    assert_eq!(
        body, want,
        "no file upload, no vocabulary [N1], and nothing stored [N10]"
    );
}

#[test]
fn the_key_goes_only_in_its_own_header_beside_the_jsons_type() {
    let req = built(Some(TRANSCRIBE), "en-US", WAV).unwrap();
    assert_eq!(field(&req, HEADER), Some(made_key()));
    let kind = field(&req, "Content-Type");
    assert_eq!(kind.as_deref(), Some("application/json"));
    assert_eq!((req.fields.len(), field(&req, "Authorization")), (2, None));
    let holds = |hay: &[u8]| {
        hay.windows(made_key().len())
            .any(|w| w == made_key().as_bytes())
    };
    assert!(!holds(&req.body), "the body holds no key");
    let others = req.fields.iter().filter(|(n, _)| n != HEADER);
    assert!(others.clone().all(|(_, v)| !holds(v)));
}

#[test]
fn the_body_is_the_wav_a_third_larger_in_a_small_wrapper() {
    let size = |model: &str, lang: &str, n: usize| {
        let wav = sound(n, 0xA5);
        built(Some(model), lang, &wav).unwrap().body.len()
    };
    for n in [0_usize, 1, 2, 3, 4, 1000] {
        let grown = n.div_ceil(3) * 4;
        let want = WRAP + TRANSCRIBE.len() + MSA.len() + grown;
        assert_eq!(size(TRANSCRIBE, MSA, n), want, "{n} bytes");
    }
    let (model, lang) = ("m".repeat(MAX_TAG_BYTES), "l".repeat(MAX_TAG_BYTES));
    let longest = size(&model, &lang, WAV.len()) - WAV.len().div_ceil(3) * 4;
    assert_eq!(
        longest,
        WRAP + 2 * MAX_TAG_BYTES,
        "the longest names the frames take"
    );
}

#[test]
fn a_run_with_no_model_or_language_or_another_shapes_call_is_out_of_range() {
    let err = |model, lang| built(model, lang, WAV).err();
    assert_eq!(err(None, MSA), Some(NetFault::Range), "no default model");
    assert_eq!(
        err(Some(TRANSCRIBE), ""),
        Some(NetFault::Range),
        "never an empty list, which would detect the language"
    );
    let form = call_to(ADDRESS, NetShape::Form(form()));
    let other = Gemini
        .build(&piece(&form, Some(TRANSCRIBE), MSA, WAV))
        .err();
    assert_eq!(other, Some(NetFault::Range));
    assert_eq!(Gemini.test(&form).err(), Some(NetFault::Range));
}

#[test]
fn a_model_or_language_that_would_break_the_json_is_written_as_a_json_string() {
    let req = built(Some("a\"b\\c"), "x\ny", WAV).unwrap();
    let (body, sound) = json_of(&req);
    assert_eq!(body["model"], "a\"b\\c");
    assert_eq!(
        body["generation_config"]["transcription_config"]["language_codes"],
        json!(["x\ny"])
    );
    assert_eq!(sound.as_deref(), Some(WAV));
}

#[test]
fn the_key_test_is_a_get_of_the_models_list_with_the_key_alone() {
    let req = Gemini.test(&gemini(TEST)).unwrap();
    assert_eq!(
        (req.verb, req.body.len(), req.fields.len()),
        (Verb::Get, 0, 1)
    );
    assert_eq!(field(&req, HEADER), Some(made_key()));
}

#[test]
fn it_keeps_the_header_a_429_waits_by() {
    let req = built(Some(TRANSCRIBE), MSA, WAV).unwrap();
    assert!(req.keep.contains(&"retry-after"));
}

#[test]
fn the_job_builds_it_and_a_piece_posts_its_json_and_reads_its_text() {
    let sending = Sending {
        shape: shape::of(ShapeCode::Gemini),
        call: gemini(ADDRESS),
        model: Some(Cow::Borrowed(TRANSCRIBE)),
        lang: Cow::Borrowed(MSA),
        tries: 1,
        pace: None,
        dropped: Arc::new(AtomicBool::new(false)),
    };
    let http = Arc::new(FakeHttp::new([reply(OK, replies::TEXT)]));
    let clock = Arc::new(Still::default());
    let got = send::piece(&kit(&http, &clock), &sending, WAV);
    assert_eq!(got.map(|t| t.to_string()), Ok(replies::SAID.to_owned()));
    let called = http.called();
    let [one] = &called[..] else {
        panic!("one call, not {}", called.len())
    };
    assert_eq!((one.url.as_str(), one.verb), (ADDRESS, Verb::Post));
    let key = one.fields.iter().find(|(n, _)| n == HEADER);
    assert_eq!(key.map(|(_, v)| v.as_slice()), Some(made_key().as_bytes()));
}

#[test]
fn a_run_whose_every_transcript_is_empty_ends_empty_not_in_a_crash() {
    let (mut b, http, _) = faked([reply(OK, replies::EMPTY), reply(OK, replies::NO_STEP_TEXT)]);
    let run = NetRun {
        model: Some(Cow::Borrowed(TRANSCRIBE)),
        lang: Cow::Borrowed(MSA),
        ..run_of(&[2, 2], 1, 1, 20)
    };
    b.ask(send(1, gemini(ADDRESS), run));
    b.ask(part(1, 0, sound(2, 1)));
    b.ask(part(1, 1, sound(2, 1)));
    let told = b.until(3).to_vec();
    assert_eq!(told.last(), Some(&fault(1, NetFault::Empty)));
    assert_eq!(http.called().len(), 2);
}
