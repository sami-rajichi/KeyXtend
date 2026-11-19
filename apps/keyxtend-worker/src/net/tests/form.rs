//! The upload form: its fields in the row's order with the model and the language, JSON asked before the sound, the
//! key only in its header, a value that would break the form refused, names escaped, a boundary in no part, and the
//! key's test a GET with the key alone.

use super::*;
use crate::net::form::{Form, escape};
use crate::net::shape::{Piece, Request, Shape};
use kx_ipc::{NetFault, NetField};
use kx_platform_net::Verb;

/// The sound the form tests send: a few bytes standing for a WAV file.
const WAV: &[u8] = b"RIFF-and-samples";
/// The form's content type, before its boundary.
const FORM_TYPE: &str = "multipart/form-data; boundary=";

/// A piece of `wav` over `call` for `model` in `lang`.
pub(super) fn piece<'a>(
    call: &'a NetCall<'a>,
    model: Option<&'a str>,
    lang: &'a str,
    wav: &'a [u8],
) -> Piece<'a> {
    Piece {
        call,
        model,
        lang,
        wav,
    }
}

/// The header `name`'s value in `req`, as text.
pub(super) fn field(req: &Request, name: &str) -> Option<String> {
    let named = req
        .fields
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name));
    named.map(|(_, v)| String::from_utf8_lossy(v).into_owned())
}

/// Where `pat` first starts in `hay`.
fn find(hay: &[u8], pat: &[u8]) -> Option<usize> {
    hay.windows(pat.len()).position(|w| w == pat)
}

/// `hay` cut at each `pat`.
fn split<'a>(mut hay: &'a [u8], pat: &[u8]) -> Vec<&'a [u8]> {
    let mut out = Vec::new();
    while let Some(at) = find(hay, pat) {
        out.push(&hay[..at]);
        hay = &hay[at + pat.len()..];
    }
    out.push(hay);
    out
}

/// `req`'s boundary, from its content type.
fn boundary(req: &Request) -> String {
    let kind = field(req, "Content-Type").unwrap();
    kind.strip_prefix(FORM_TYPE).unwrap().to_owned()
}

/// Each part of `req`'s body: its name, its file name when it has one, its head and its bytes.
pub(super) fn parts(req: &Request) -> Vec<(String, Option<String>, String, Vec<u8>)> {
    let delim = format!("\r\n--{}", boundary(req));
    let body = [&b"\r\n"[..], &req.body].concat();
    let chunks = split(&body, delim.as_bytes());
    assert_eq!(
        chunks.first().map(|c| c.len()),
        Some(0),
        "starts with a boundary"
    );
    assert_eq!(
        chunks.last().copied(),
        Some(&b"--\r\n"[..]),
        "ends with the last"
    );
    let part = |chunk: &[u8]| {
        let chunk = chunk.strip_prefix(b"\r\n").unwrap();
        let at = find(chunk, b"\r\n\r\n").unwrap();
        let head = String::from_utf8(chunk[..at].to_vec()).unwrap();
        let quoted = |key: &str| {
            let rest = head.split(&format!("; {key}=\"")).nth(1)?;
            rest.split('"').next().map(str::to_owned)
        };
        (
            quoted("name").unwrap(),
            quoted("filename"),
            head.clone(),
            chunk[at + 4..].to_vec(),
        )
    };
    chunks[1..chunks.len() - 1]
        .iter()
        .map(|c| part(c))
        .collect()
}

/// The part names in `req`'s body, in order.
pub(super) fn names(req: &Request) -> Vec<String> {
    parts(req).into_iter().map(|p| p.0).collect()
}

#[test]
fn the_form_sends_the_model_and_the_language_then_json_asked_then_the_sound() {
    let call = call();
    let req = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).unwrap();
    assert_eq!(req.verb, Verb::Post);
    let got = parts(&req);
    assert_eq!(
        names(&req),
        ["model", "language", "response_format", "file"]
    );
    assert_eq!(got[0].3, MODEL.as_bytes());
    assert_eq!(got[1].3, LANG.as_bytes());
    assert_eq!(got[2].3, b"json", "no word or character timings");
    assert!(got[..3].iter().all(|p| p.1.is_none()));
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
fn a_list_language_and_the_rows_fixed_fields_keep_the_rows_order() {
    let keep = r#"{"data_collection":"deny","zdr":true}"#;
    let fixed = vec![
        NetField {
            name: Cow::Borrowed("provider"),
            value: Cow::Borrowed(keep),
        },
        NetField {
            name: Cow::Borrowed("temperature"),
            value: Cow::Borrowed("0"),
        },
    ];
    let shape = NetShape::Form(NetForm {
        lang: Cow::Borrowed("languages"),
        list: true,
        fixed,
        paywall: false,
        region: None,
    });
    let call = call_to(URL, shape);
    let req = Form.build(&piece(&call, Some(MODEL), "ar", WAV)).unwrap();
    let want = [
        "model",
        "languages[]",
        "provider",
        "temperature",
        "response_format",
        "file",
    ];
    assert_eq!(names(&req), want);
    let got = parts(&req);
    assert_eq!(
        (got[1].3.as_slice(), got[2].3.as_slice()),
        (&b"ar"[..], keep.as_bytes())
    );
}

#[test]
fn a_run_without_a_model_sends_no_model_field() {
    let call = call();
    let req = Form.build(&piece(&call, None, LANG, WAV)).unwrap();
    assert_eq!(names(&req), ["language", "response_format", "file"]);
}

#[test]
fn the_key_goes_only_in_its_header() {
    let call = call();
    let req = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).unwrap();
    assert_eq!(
        field(&req, "Authorization"),
        Some(format!("Bearer {KEY_TEXT}"))
    );
    assert_eq!(find(&req.body, KEY), None, "the body holds no key");
    let others = req.fields.iter().filter(|(n, _)| n != "Authorization");
    assert!(others.clone().all(|(_, v)| find(v, KEY).is_none()));
    let mut call = call;
    call.header = NetHeader::Named(Cow::Borrowed("xi-api-key"));
    let req = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).unwrap();
    assert_eq!(field(&req, "xi-api-key"), Some(KEY_TEXT.to_owned()));
    assert_eq!(field(&req, "Authorization"), None);
}

#[test]
fn a_value_that_would_break_the_form_is_refused() {
    let call = call();
    for bad in ["a\r\nb", "a\nb", "a\rb", "a\0b"] {
        let got = Form.build(&piece(&call, Some(bad), LANG, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "model {bad:?}");
        let got = Form.build(&piece(&call, Some(MODEL), bad, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "language {bad:?}");
    }
    let fixed = vec![NetField {
        name: Cow::Borrowed("prompt"),
        value: Cow::Borrowed("a\r\n--x"),
    }];
    let call = call_to(URL, NetShape::Form(NetForm { fixed, ..form() }));
    let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
    assert_eq!(got, Some(NetFault::Range), "a fixed value");
}

#[test]
fn a_fixed_field_may_not_repeat_the_forms_own_names() {
    for name in ["model", "file", "response_format", "language", "MODEL"] {
        let fixed = vec![NetField {
            name: Cow::Borrowed(name),
            value: Cow::Borrowed("x"),
        }];
        let call = call_to(URL, NetShape::Form(NetForm { fixed, ..form() }));
        let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "{name}");
    }
}

#[test]
fn a_fixed_field_may_not_repeat_a_list_languages_name() {
    for name in ["languages[]", "LANGUAGES[]", "languages"] {
        let fixed = vec![NetField {
            name: Cow::Borrowed(name),
            value: Cow::Borrowed("x"),
        }];
        let shape = NetShape::Form(NetForm {
            lang: Cow::Borrowed("languages"),
            list: true,
            fixed,
            paywall: false,
            region: None,
        });
        let call = call_to(URL, shape);
        let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
        assert_eq!(got, Some(NetFault::Range), "{name}");
    }
}

#[test]
fn names_are_escaped_as_forms_escape_them() {
    assert_eq!(escape("a\"b\r\nc"), "a%22b%0D%0Ac");
    assert!(matches!(escape("model"), Cow::Borrowed("model")));
}

#[test]
fn the_boundary_is_in_no_part_even_when_the_sound_holds_another() {
    let call = call();
    let first = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).unwrap();
    let taken = boundary(&first);
    let wav = [b"--".as_slice(), taken.as_bytes(), b"\r\n"].concat();
    let req = Form.build(&piece(&call, Some(MODEL), LANG, &wav)).unwrap();
    let other = boundary(&req);
    assert_ne!(other, taken);
    assert_eq!(find(&wav, other.as_bytes()), None);
    assert_eq!(parts(&req)[3].3, wav);
    let again = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).unwrap();
    assert_eq!(
        boundary(&again),
        taken,
        "the same piece makes the same boundary"
    );
}

#[test]
fn the_form_keeps_the_headers_a_429_is_read_by() {
    let call = call();
    let req = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).unwrap();
    for name in [
        "retry-after",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-reset-requests",
    ] {
        assert!(req.keep.contains(&name), "{name}");
    }
}

#[test]
fn the_key_test_is_a_get_of_the_test_address_with_the_key_alone() {
    let call = call_to(MODELS, NetShape::Form(form()));
    let req = Form.test(&call).unwrap();
    assert_eq!(
        (req.verb, req.body.len(), req.fields.len()),
        (Verb::Get, 0, 1)
    );
    assert_eq!(
        field(&req, "Authorization"),
        Some(format!("Bearer {KEY_TEXT}"))
    );
}

#[test]
fn another_shapes_call_is_out_of_range() {
    let call = call_to(URL, NetShape::Eleven(super::eleven::own_form()));
    let got = Form.build(&piece(&call, Some(MODEL), LANG, WAV)).err();
    assert_eq!(got, Some(NetFault::Range));
}
