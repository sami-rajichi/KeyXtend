//! The upload form in each variant its rows take, with made-up models and fields at one example address: a list field
//! and a region code, a paywall, no model, a keep-nothing field on and off. Each sends its fields in order with the key
//! in its header, and `replies` reads each service's faults from its fixtures; kx-mod-voice pins the rows themselves.

mod replies;

use super::form::{names, parts, piece};
use super::*;
use crate::net::form::Form;
use crate::net::send::{self, Sending};
use crate::net::shape::{self, Shape};
use kx_ipc::{NetField, ShapeCode};
use kx_platform_net::Verb;
use std::sync::atomic::AtomicBool;

/// The one address the row tests send to.
const ADDRESS: &str = "https://stt.example.com/v1/audio/transcriptions";
/// The sound the row tests send.
const WAV: &[u8] = b"RIFF-and-samples";
/// What the rows' text fixtures say.
const SAID: &str = "Bonjour tout le monde.";
/// `OpenAI`'s error code for a country or region it does not serve [O5].
const REGION: &str = "unsupported_country_region_territory";

/// A form row's parameters: the service it stands for, its language field, list flag, fixed fields, paywall and region
/// code.
#[derive(Clone, Copy)]
struct Row {
    name: &'static str,
    lang: &'static str,
    list: bool,
    fixed: &'static [(&'static str, &'static str)],
    paywall: bool,
    region: Option<&'static str>,
}

/// Service `name`'s row with the one `language` field and no reply rule.
const fn plain(name: &'static str) -> Row {
    Row {
        name,
        lang: "language",
        list: false,
        fixed: &[],
        paywall: false,
        region: None,
    }
}

/// A made-up keep-nothing field while its switch is on, as `OpenRouter`'s row adds one [R2][R5].
const ZDR: (&str, &str) = ("keep", r#"{"none":true}"#);

/// Groq [G1].
const GROQ: Row = plain("groq");
/// `OpenAI`'s variant: a made-up list field in place of `language` [O2], and a country refusal with its code [O5].
const OPENAI: Row = Row {
    lang: "langs",
    list: true,
    region: Some(REGION),
    ..plain("openai")
};
/// Mistral: multipart is the working reading of its guide [M1].
const MISTRAL: Row = plain("mistral");
/// Together: its 403 is a plan limit or a missing payment method [T5].
const TOGETHER: Row = Row {
    paywall: true,
    ..plain("together")
};
/// Lemonfox [L2].
const LEMONFOX: Row = plain("lemonfox");
/// `DeepInfra` [D3].
const DEEPINFRA: Row = plain("deepinfra");
/// `OpenRouter` with its keep-nothing switch on: its 403 is a key's spending cap [R2][R7].
const ROUTER: Row = Row {
    fixed: &[ZDR],
    paywall: true,
    ..plain("openrouter")
};
/// `OpenRouter` with its keep-nothing switch off.
const ROUTER_OPEN: Row = Row {
    fixed: &[],
    ..ROUTER
};
/// A Custom address the user types.
const CUSTOM: Row = plain("custom");

impl Row {
    /// The row's call to the test address, its key as a bearer.
    fn call(self) -> NetCall<'static> {
        let fixed = self.fixed.iter().map(|&(name, value)| NetField {
            name: Cow::Borrowed(name),
            value: Cow::Borrowed(value),
        });
        let form = NetForm {
            lang: Cow::Borrowed(self.lang),
            list: self.list,
            fixed: fixed.collect(),
            paywall: self.paywall,
            region: self.region.map(Cow::Borrowed),
        };
        call_to(ADDRESS, NetShape::Form(form))
    }
}

/// One row's request: the model it names, the code it sends and the parts in order, with their values.
struct Case {
    row: Row,
    model: Option<&'static str>,
    lang: &'static str,
    parts: &'static [(&'static str, &'static str)],
}

/// Each row with a made-up model; Lemonfox names none and wants an English name [L2], a Custom row a typed id.
const CASES: [Case; 9] = [
    case(GROQ, Some("model-large-v3-turbo"), "fr", &[]),
    case(OPENAI, Some("transcribe"), "fr", &[]),
    case(MISTRAL, Some("model-mini-transcribe-3-0"), "ar", &[]),
    case(TOGETHER, Some("vendor/model-large-v3"), "en", &[]),
    case(LEMONFOX, None, "french", &[]),
    case(DEEPINFRA, Some(VENDOR_MODEL), "fr", &[]),
    case(ROUTER, Some(VENDOR_MODEL), "fr", &[ZDR]),
    case(ROUTER_OPEN, Some(VENDOR_MODEL), "fr", &[]),
    case(CUSTOM, Some("my-own-model"), "fr", &[]),
];
/// A made-up model under a vendor's name, as some rows offer.
const VENDOR_MODEL: &str = "vendor/model-large-v3-turbo";

/// A case for `row`, naming `model` in `lang`, with `fixed` after the language.
const fn case(
    row: Row,
    model: Option<&'static str>,
    lang: &'static str,
    fixed: &'static [(&'static str, &'static str)],
) -> Case {
    Case {
        row,
        model,
        lang,
        parts: fixed,
    }
}

impl Case {
    /// The parts the request holds, in order: the model, the language, the fixed fields, JSON asked, the sound.
    fn want(&self) -> Vec<(String, Vec<u8>)> {
        let lang = if self.row.list {
            format!("{}[]", self.row.lang)
        } else {
            self.row.lang.to_owned()
        };
        let model = self
            .model
            .map(|m| ("model".to_owned(), m.as_bytes().to_vec()));
        let fixed = (self.parts.iter()).map(|(n, v)| ((*n).to_owned(), v.as_bytes().to_vec()));
        let tail = [
            ("response_format".to_owned(), b"json".to_vec()),
            ("file".to_owned(), WAV.to_vec()),
        ];
        let lang = (lang, self.lang.as_bytes().to_vec());
        model
            .into_iter()
            .chain([lang])
            .chain(fixed)
            .chain(tail)
            .collect()
    }

    /// What one piece's thread sends with.
    fn sending(&self) -> Sending {
        Sending {
            shape: shape::of(ShapeCode::Form),
            call: self.row.call(),
            model: self.model.map(Cow::Borrowed),
            lang: Cow::Borrowed(self.lang),
            tries: 1,
            pace: None,
            dropped: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[test]
fn each_row_sends_its_model_and_language_then_its_fixed_fields_then_the_sound() {
    for c in &CASES {
        let call = c.row.call();
        let req = Form.build(&piece(&call, c.model, c.lang, WAV)).unwrap();
        let got: Vec<(String, Vec<u8>)> = parts(&req).into_iter().map(|p| (p.0, p.3)).collect();
        assert_eq!(got, c.want(), "{}", c.row.name);
    }
}

#[test]
fn openrouters_keep_nothing_field_goes_only_while_its_switch_is_on() {
    let model = Some(VENDOR_MODEL);
    let named = |row: Row| names(&Form.build(&piece(&row.call(), model, "fr", WAV)).unwrap());
    assert!(named(ROUTER).contains(&ZDR.0.to_owned()));
    assert!(!named(ROUTER_OPEN).contains(&ZDR.0.to_owned()));
}

#[test]
fn each_row_posts_its_form_with_the_key_in_its_header_and_reads_its_text() {
    let texts = replies::TEXTS;
    for (c, text) in CASES.iter().zip(texts) {
        let http = Arc::new(FakeHttp::new([reply(OK, text)]));
        let clock = Arc::new(Still::default());
        let got = send::piece(&kit(&http, &clock), &c.sending(), WAV);
        let got = got.map(|t| t.trim().to_owned());
        assert_eq!(got, Ok(SAID.to_owned()), "{}", c.row.name);
        let called = http.called();
        let [one] = &called[..] else {
            panic!("{}: one call, not {}", c.row.name, called.len())
        };
        assert_eq!((one.url.as_str(), one.verb), (ADDRESS, Verb::Post));
        let auth = one.fields.iter().find(|(n, _)| n == "Authorization");
        let bearer = format!("Bearer {}", made_key()).into_bytes();
        assert_eq!(auth.map(|(_, v)| v), Some(&bearer), "{}", c.row.name);
    }
}
