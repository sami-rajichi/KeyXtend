//! `Http::call` over the fake: a POST's verb, headers and body sent as given, a head that would break refused before
//! anything is sent, kept headers matched in any case, a redirect handed back unfollowed, and no value in any `Debug`.

use crate::fake::{Act, FakeHttp, Step};
use crate::{Call, Field, Head, Http, NetError, Target, Verb, wait_ms};

/// The address the calls go to.
const URL: &str = "https://api.example/v1/audio";
/// A made-up key, never a real one; no test sends it anywhere.
const KEY: &[u8] = b"Bearer kx-test-key-not-real";
/// The stall time the calls ask with.
const STALL_S: u16 = 30;
/// The status of a reply that holds what was asked.
const OK: u16 = 200;

fn target() -> Target {
    Target::parse(URL).unwrap()
}

/// A call of `verb` to `target` with `fields`, `body` and `keep`.
fn call<'a>(
    target: &'a Target,
    verb: Verb,
    fields: &'a [Field<'a>],
    body: &'a [u8],
    keep: &'a [&'a str],
) -> Call<'a> {
    Call {
        target,
        verb,
        fields,
        body,
        keep,
        stall_s: STALL_S,
    }
}

/// The key's header and a form's type.
fn fields() -> [Field<'static>; 2] {
    [
        Field {
            name: "Authorization",
            value: KEY,
        },
        Field {
            name: "Content-Type",
            value: b"multipart/form-data; boundary=b",
        },
    ]
}

#[test]
fn a_post_sends_its_verb_headers_and_body_as_given() {
    let (http, t, fields) = (FakeHttp::new([Act::with(OK, b"{}")]), target(), fields());
    let reply = http.call(&call(&t, Verb::Post, &fields, b"body", &[]));
    assert_eq!(reply.unwrap().head.status, OK);
    let called = http.called();
    assert_eq!(called.len(), 1);
    assert_eq!((called[0].url.as_str(), called[0].verb), (URL, Verb::Post));
    let names: Vec<_> = called[0].fields.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["Authorization", "Content-Type"]);
    assert_eq!(called[0].fields[0].1, KEY);
    assert_eq!(
        (called[0].body.as_slice(), called[0].stall_s),
        (&b"body"[..], STALL_S)
    );
}

#[test]
fn a_head_that_would_break_is_refused_before_anything_is_sent() {
    let bad: [(&str, &[u8]); 7] = [
        ("Bad Name", b"v"),
        ("", b"v"),
        ("X:y", b"v"),
        ("X-Key", b"a\r\nX-Evil: 1"),
        ("X-Key", b"a\nb"),
        ("X-Key", b"\0"),
        ("X-Key", "\u{e9}".as_bytes()),
    ];
    let (http, t) = (FakeHttp::new([]), target());
    for (name, value) in bad {
        let fields = [Field { name, value }];
        let got = http.call(&call(&t, Verb::Post, &fields, b"", &[]));
        assert_eq!(got.err(), Some(NetError::Header), "{name:?}");
    }
    let got = http.call(&call(&t, Verb::Post, &[], b"", &["no good"]));
    assert_eq!(got.err(), Some(NetError::Header), "a kept name is a token");
    assert!(http.called().is_empty());
}

#[test]
fn a_get_carries_headers_and_never_a_body() {
    let (http, t, fields) = (FakeHttp::new([Act::with(OK, b"{}")]), target(), fields());
    let got = http.call(&call(&t, Verb::Get, &fields, b"x", &[]));
    assert_eq!(got.err(), Some(NetError::Header));
    let got = http.call(&call(&t, Verb::Get, &fields, b"", &[]));
    assert_eq!(got.unwrap().head.status, OK);
    assert_eq!(http.called().len(), 1);
    assert_eq!(http.called()[0].verb, Verb::Get);
}

#[test]
fn only_the_kept_headers_come_back_matched_in_any_case() {
    let head = Head {
        status: OK,
        extra: vec![
            ("Retry-After".to_owned(), "7".to_owned()),
            ("X-Other".to_owned(), "1".to_owned()),
        ],
        ..Head::default()
    };
    let (http, t) = (FakeHttp::new([Act::Answer(head, Vec::new())]), target());
    let reply = http.call(&call(&t, Verb::Post, &[], b"", &["retry-after"]));
    let head = reply.unwrap().head;
    assert_eq!(head.header("RETRY-AFTER"), Some("7"));
    assert_eq!(head.header("x-other"), None);
}

#[test]
fn a_redirect_is_handed_back_and_never_followed() {
    let acts = [
        Act::moved("https://evil.example/take"),
        Act::with(OK, b"{}"),
    ];
    let (http, t) = (FakeHttp::new(acts), target());
    let reply = http.call(&call(&t, Verb::Post, &fields(), b"sound", &[]));
    assert_eq!(reply.unwrap().head.status, Head::moved("").status);
    assert_eq!(
        http.called().len(),
        1,
        "the key and the body went once, to one host"
    );
}

#[test]
fn a_quiet_answer_past_the_wait_is_a_stall_and_a_refused_link_is_net() {
    let quiet = Act::Answer(Head::whole(1), vec![Step::Quiet(wait_ms(STALL_S))]);
    let (http, t) = (FakeHttp::new([quiet, Act::Fail(NetError::Net)]), target());
    let mut reply = http.call(&call(&t, Verb::Post, &[], b"", &[])).unwrap();
    assert_eq!(reply.body.read(&mut [0; 4]), Err(NetError::Stall));
    let got = http.call(&call(&t, Verb::Post, &[], b"", &[]));
    assert_eq!(got.err(), Some(NetError::Net));
}

#[test]
fn no_debug_print_or_error_shows_a_header_value_or_the_body() {
    let (http, t, fields) = (FakeHttp::new([]), target(), fields());
    let sent = call(&t, Verb::Post, &fields, b"secret sound", &["retry-after"]);
    let shown = format!("{sent:?} {:?}", fields[0]);
    assert!(
        shown.contains("Authorization") && shown.contains("retry-after"),
        "{shown}"
    );
    let key = String::from_utf8_lossy(KEY);
    assert!(
        !shown.contains(&*key) && !shown.contains("secret sound"),
        "{shown}"
    );
    assert!(!shown.contains(URL), "{shown}");
    let err = http.call(&sent).unwrap_err();
    assert!(!err.to_string().contains(&*key));
    let called = format!("{:?}", http.called());
    assert!(
        !called.contains(&*key) && !called.contains("secret sound"),
        "{called}"
    );
    assert!(!NetError::Header.to_string().contains(&*key));
}
