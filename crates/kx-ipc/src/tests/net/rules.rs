//! The net rules and caps the port shares: a service's address is plain HTTPS naming one host, request names and form
//! values hold nothing a request misreads, a piece fits the row's size and time, and the text and reply caps.

use super::*;
use crate::IpcError::BadField;
use crate::rules::{form_value, https, https_host, token};
use crate::{
    MAX_NET_REPLY_BYTES, MAX_RECORD_BYTES, MAX_TEXT_BYTES, NET_JOB, NET_MAX_MB, PIECE_S,
    RECORD_MAX_S, SOUND_HZ, piece_bytes,
};

/// Seconds in a day.
const DAY_S: u32 = 24 * 60 * 60;

#[test]
fn an_https_address_passes_and_names_its_host() {
    let azure = "https://kx-test.cognitiveservices.azure.com/speechtotext/transcriptions:transcribe?api-version=2025-10-15";
    let good = [
        (GROQ, "api.groq.com"),
        (azure, "kx-test.cognitiveservices.azure.com"),
        ("https://192.168.1.20:8443/v1", "192.168.1.20"),
        ("https://localhost:8000", "localhost"),
        (
            "https://api.elevenlabs.io/v1/speech-to-text?x=1",
            "api.elevenlabs.io",
        ),
    ];
    for (url, host) in good {
        assert_eq!(https(url), Ok(()), "{url}");
        assert_eq!(https_host(url), host);
    }
}

#[test]
fn an_address_that_could_reach_another_host_or_break_a_request_is_refused() {
    let bad = [
        "",
        "http://api.groq.com/v1",
        "HTTPS://api.groq.com/v1",
        "//api.groq.com/v1",
        "https://",
        "https:///v1",
        "https://api.groq.com@evil.example/v1",
        "https://user:pass@api.groq.com/v1",
        "https://API.groq.com/v1",
        "https://[::1]/v1",
        "https://.groq.com/v1",
        "https://api..groq.com/v1",
        "https://api.groq.com:0/v1",
        "https://api.groq.com:99999/v1",
        "https://api.groq.com:/v1",
        "https://api.groq.com:+1/v1",
        "https://api.groq.com/v1#part",
        "https://api.groq.com/a b",
        "https://api.groq.com/v1\r\nX-Key: 1",
        "https://مثال.com/v1",
    ];
    for url in bad {
        assert_eq!(https(url), Err(BadField), "{url:?}");
    }
    let long_host = format!("{HTTPS}{}/v1", "h".repeat(crate::MAX_HOST_BYTES + 1));
    assert_eq!(https(&long_host), Err(BadField));
}

#[test]
fn a_request_name_is_letters_digits_and_two_marks() {
    let good = [
        "language",
        "languages",
        "model_id",
        "xi-api-key",
        "Ocp-Apim-Subscription-Key",
    ];
    for name in good {
        assert_eq!(token(name), Ok(()), "{name}");
    }
    for name in ["a b", "a.b", "a:b", "a[]", "x\r\nkey", "clé", "a\0"] {
        assert_eq!(token(name), Err(BadField), "{name:?}");
    }
}

#[test]
fn a_form_value_is_visible_ascii_and_spaces() {
    for value in [r#"{"zdr":true}"#, "a b", ""] {
        assert_eq!(form_value(value), Ok(()), "{value}");
    }
    for value in ["a\r\nb", "a\tb", "é", "\x7f"] {
        assert_eq!(form_value(value), Err(BadField), "{value:?}");
    }
}

#[test]
fn a_piece_fits_both_the_rows_size_and_its_time_in_whole_samples() {
    let per_s = SOUND_HZ * SAMPLE_BYTES;
    let mib = 1 << 20;
    assert_eq!(piece_bytes(25, 3_600), 25 * mib, "Groq: size first");
    assert_eq!(piece_bytes(1, 10), 10 * per_s, "time first");
    let top = piece_bytes(*NET_MAX_MB.end(), *NET_MAX_S.end());
    assert_eq!(
        top, MAX_RECORD_BYTES,
        "the longest piece is the longest dictation"
    );
    for (mb, s) in [(1, 10), (7, 333), (25, 3_600)] {
        assert_eq!(piece_bytes(mb, s) % SAMPLE_BYTES, 0);
    }
}

#[test]
fn the_caps_are_the_plans_and_hold_each_other() {
    assert_eq!(NET_JOB, "speech-net", "ADR-0047 decision 1");
    let longest = RECORD_MAX_S.end() / u16::from(*PIECE_S.end());
    assert_eq!(MAX_NET_TEXT_BYTES, MAX_TEXT_BYTES * usize::from(longest));
    const { assert!(MAX_NET_REPLY_BYTES >= MAX_NET_TEXT_BYTES) };
    assert_eq!(*NET_MAX_S.start(), u16::from(*PIECE_S.start()));
    assert_eq!(*NET_MAX_S.end(), *RECORD_MAX_S.end());
    assert_eq!(
        NET_RENEW_S,
        1..=31 * DAY_S,
        "a month, the longest allowance a service names"
    );
}
