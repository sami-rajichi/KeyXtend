//! `ElevenLabs`' own form [E3]: `model_id`, the row's language and fixed fields, then the piece as `file`, nothing
//! else; the key only in the call's own header; a run with no model, another shape's call or a repeated name refused;
//! the key's test a GET with the key alone; and a piece posted and read end to end. `replies` reads each answer as its
//! error page says [E8].

mod replies;

use super::form::{field, names, parts, piece};
use super::*;
use crate::net::eleven::Eleven;
use crate::net::send::{self, Sending};
use crate::net::shape::{self, Shape};
use kx_ipc::{NetFault, NetField, ShapeCode};
use kx_platform_net::Verb;
use std::sync::atomic::AtomicBool;

/// The one address the tests send to, never the table's.
const ADDRESS: &str = "https://stt.example.com/v1/speech-to-text";
/// The one key test address.
const TEST: &str = "https://stt.example.com/v1/user/subscription";
/// The header the row names for the key [E3].
const HEADER: &str = "xi-api-key";
/// A model the row offers, as the run names it [E4].
const SCRIBE: &str = "scribe_v3";
/// The sound the tests send.
const WAV: &[u8] = b"RIFF-and-samples";
/// What the text fixture says.
const SAID: &str = "Bonjour tout le monde.";
/// The field that asks for no sound tags.
const TAGS: &str = "tag_audio_events";

/// The row's form: its language field, and no sound tags such as "(laughter)" in the text.
fn own_form() -> NetForm<'static> {
    NetForm {
        lang: Cow::Borrowed("language_code"),
        fixed: vec![NetField {
            name: Cow::Borrowed(TAGS),
            value: Cow::Borrowed("false"),
        }],
        ..form()
    }
}

/// A call to `url` in `ElevenLabs`' shape with `form`, the key under its own header.
fn eleven_with(url: &'static str, form: NetForm<'static>) -> NetCall<'static> {
    NetCall {
        header: NetHeader::Named(Cow::Borrowed(HEADER)),
        ..call_to(url, NetShape::Eleven(form))
    }
}

/// A call to `url` in `ElevenLabs`' shape with the row's form.
fn eleven(url: &'static str) -> NetCall<'static> {
    eleven_with(url, own_form())
}

#[test]
fn the_form_sends_the_model_id_its_language_and_fixed_fields_then_the_sound_and_nothing_else() {
    let call = eleven(ADDRESS);
    let req = Eleven
        .build(&piece(&call, Some(SCRIBE), "fr", WAV))
        .unwrap();
    assert_eq!(req.verb, Verb::Post);
    assert_eq!(names(&req), ["model_id", "language_code", TAGS, "file"]);
    let got = parts(&req);
    let values: Vec<&[u8]> = got[..3].iter().map(|p| p.3.as_slice()).collect();
    assert_eq!(values, [SCRIBE.as_bytes(), b"fr", b"false"]);
    assert!(got[..3].iter().all(|p| p.1.is_none()), "three text fields");
    assert_eq!(
        (got[3].1.as_deref(), got[3].3.as_slice()),
        (Some("piece.wav"), WAV)
    );
    assert!(
        got[3].2.contains("\r\nContent-Type: audio/wav"),
        "{}",
        got[3].2
    );
}

#[test]
fn a_fixed_field_that_repeats_one_of_its_own_names_is_out_of_range() {
    for name in ["model_id", "Language_Code", "file"] {
        let mut form = own_form();
        form.fixed[0].name = Cow::Borrowed(name);
        let call = eleven_with(ADDRESS, form);
        let got = Eleven.build(&piece(&call, Some(SCRIBE), "fr", WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "{name}");
    }
    let listed = NetForm {
        list: true,
        ..own_form()
    };
    let call = eleven_with(ADDRESS, listed);
    let req = Eleven
        .build(&piece(&call, Some(SCRIBE), "fr", WAV))
        .unwrap();
    assert_eq!(names(&req)[1], "language_code[]", "a list as the row says");
}

#[test]
fn the_key_goes_only_in_its_own_header() {
    let call = eleven(ADDRESS);
    let req = Eleven
        .build(&piece(&call, Some(SCRIBE), "en", WAV))
        .unwrap();
    assert_eq!(field(&req, HEADER), Some(KEY_TEXT.to_owned()));
    assert_eq!(field(&req, "Authorization"), None);
    let holds = |hay: &[u8]| hay.windows(KEY.len()).any(|w| w == KEY);
    assert!(!holds(&req.body), "the body holds no key");
    let others = req.fields.iter().filter(|(n, _)| n != HEADER);
    assert!(others.clone().all(|(_, v)| !holds(v)));
}

#[test]
fn a_run_with_no_model_another_shapes_call_or_a_breaking_value_is_out_of_range() {
    let call = eleven(ADDRESS);
    let got = Eleven.build(&piece(&call, None, "fr", WAV)).err();
    assert_eq!(got, Some(NetFault::Range), "the shape has no default model");
    let form = call_to(ADDRESS, NetShape::Form(form()));
    assert_eq!(
        Eleven.build(&piece(&form, Some(SCRIBE), "fr", WAV)).err(),
        Some(NetFault::Range)
    );
    assert_eq!(Eleven.test(&form).err(), Some(NetFault::Range));
    for bad in ["a\r\nb", "a\nb", "a\0b"] {
        let got = Eleven.build(&piece(&call, Some(bad), "fr", WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "model {bad:?}");
        let got = Eleven.build(&piece(&call, Some(SCRIBE), bad, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "language {bad:?}");
    }
}

#[test]
fn the_key_test_is_a_get_of_the_test_address_with_the_key_alone() {
    let req = Eleven.test(&eleven(TEST)).unwrap();
    assert_eq!(
        (req.verb, req.body.len(), req.fields.len()),
        (Verb::Get, 0, 1)
    );
    assert_eq!(field(&req, HEADER), Some(KEY_TEXT.to_owned()));
}

#[test]
fn it_keeps_the_header_a_429_waits_by() {
    let req = Eleven
        .build(&piece(&eleven(ADDRESS), Some(SCRIBE), "fr", WAV))
        .unwrap();
    assert!(req.keep.contains(&"retry-after"));
}

#[test]
fn the_job_builds_it_and_a_piece_posts_its_form_and_reads_its_text() {
    let sending = Sending {
        shape: shape::of(ShapeCode::Eleven).expect("built"),
        call: eleven(ADDRESS),
        model: Some(Cow::Borrowed(SCRIBE)),
        lang: Cow::Borrowed("fr"),
        tries: 1,
        pace: None,
        dropped: Arc::new(AtomicBool::new(false)),
    };
    let http = Arc::new(FakeHttp::new([reply(OK, replies::TEXT)]));
    let clock = Arc::new(Still::default());
    let got = send::piece(&kit(&http, &clock), &sending, WAV);
    assert_eq!(got.map(|t| t.to_string()), Ok(SAID.to_owned()));
    let called = http.called();
    let [one] = &called[..] else {
        panic!("one call, not {}", called.len())
    };
    assert_eq!((one.url.as_str(), one.verb), (ADDRESS, Verb::Post));
    let key = one.fields.iter().find(|(n, _)| n == HEADER);
    assert_eq!(key.map(|(_, v)| v.as_slice()), Some(KEY));
}
