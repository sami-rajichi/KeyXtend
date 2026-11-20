//! The net rules and caps the port shares: a service's address is plain HTTPS naming one host that never reads as a
//! number, request names, form values, model ids and language codes hold nothing a request misreads, and a model header
//! names no header a request sets. A piece fits the row's size and time, three shapes need a model, and the caps hold.

use super::*;
use crate::IpcError::BadField;
use crate::code::Code;
use crate::rules::{
    AUTHORIZATION, CONTENT_TYPE, form_value, https, https_host, https_origin, model, token,
    unset_header,
};
use crate::{
    MAX_NET_REPLY_BYTES, MAX_NET_RUN_TEXT_BYTES, MAX_RECORD_BYTES, MAX_TEXT_BYTES, NET_JOB,
    NET_MAX_MB, PIECE_S, RECORD_MAX_S, SOUND_HZ, ShapeCode, piece_bytes,
};

/// Seconds in a day.
const DAY_S: u32 = 24 * 60 * 60;

#[test]
fn an_https_address_passes_and_names_its_host() {
    let colon =
        "https://kx-test.speech.example.com/v1/transcriptions:transcribe?api-version=2025-10-15";
    let good = [
        (GROQ, "api.groq.com"),
        (colon, "kx-test.speech.example.com"),
        ("https://stt.lan:8443/v1", "stt.lan"),
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
fn an_addresss_origin_is_its_scheme_host_and_port_up_to_its_path_or_query() {
    for (url, origin) in [
        ("https://stt.lan:8443/v1?x=1", "https://stt.lan:8443"),
        ("https://api.groq.com?x=1", "https://api.groq.com"),
        ("https://localhost:8000", "https://localhost:8000"),
        ("https://api.groq.com/v 1", "https://api.groq.com"),
        ("http://api.groq.com/v1", ""),
    ] {
        assert_eq!(https_origin(url), origin, "{url}");
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
fn a_host_whose_last_label_reads_as_a_number_is_refused() {
    let numbers = [
        "https://2130706433/v1",
        "https://0x7f.1/v1",
        "https://127.0.0.1/v1",
        "https://192.168.1.20:8443/v1",
        "https://a.0x7f/v1",
        "https://a.0x/v1",
        "https://a.0177/v1",
        "https://speech.example.4/v1",
    ];
    for url in numbers {
        assert_eq!(https(url), Err(BadField), "{url}");
    }
    let names = [
        "https://0x7f.speech/v1",
        "https://1.example/v1",
        "https://a.0xg/v1",
        "https://a.x7f/v1",
        "https://a.4b/v1",
    ];
    for url in names {
        assert_eq!(https(url), Ok(()), "{url}");
    }
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
fn a_model_id_is_a_name_with_dots_slashes_and_colons_and_a_language_a_name() {
    let ids = [
        "whisper-large-v3-turbo",
        "gpt-transcribe",
        "openai/whisper-large-v3",
        "voxtral-mini-transcribe-3-0",
        "scribe_v3",
        "gemini-3.5-transcribe",
        "MAI-Transcribe-2",
        "whisper:large",
    ];
    for id in ids {
        assert_eq!(model(id), Ok(()), "{id}");
    }
    let bad = [
        "", "a b", "a\"b", "a\\b", "a?b", "a#b", "a%20b", "a&b", "a=b", "a+b", "/a", "a/", "a//b",
        "../a", "a/../b", "a/./b", ".", "..", "عربي", "a\nb",
    ];
    for id in bad {
        assert_eq!(model(id), Err(BadField), "{id:?}");
    }
    for lang in ["fr", "ar-SA", "french", "zh_Hans"] {
        assert_eq!(token(lang), Ok(()), "{lang}");
    }
    for lang in [
        "fr FR", "fr/FR", "fr.FR", "fr:FR", "fr\"", "fr?", "fr#", "fr%", "fr\\",
    ] {
        assert_eq!(token(lang), Err(BadField), "{lang:?}");
    }
}

#[test]
fn elevenlabs_gemini_and_assemblyai_alone_need_a_model() {
    for &shape in ShapeCode::ALL {
        let needs = matches!(
            shape,
            ShapeCode::Eleven | ShapeCode::Gemini | ShapeCode::Assembly
        );
        assert_eq!(shape.needs_model(), needs, "{shape:?}");
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
    assert_eq!(piece_bytes(10, 600), 10 * mib, "size first");
    assert_eq!(piece_bytes(1, 10), 10 * per_s, "time first");
    let top = piece_bytes(*NET_MAX_MB.end(), *NET_MAX_S.end());
    assert_eq!(top, 600 * per_s, "the longest piece is ten minutes");
    assert!(top < MAX_RECORD_BYTES, "a long dictation goes in pieces");
    for (mb, s) in [(1, 10), (7, 333), (25, 600)] {
        assert_eq!(piece_bytes(mb, s) % SAMPLE_BYTES, 0);
    }
}

#[test]
fn the_caps_are_the_plans_and_hold_each_other() {
    assert_eq!(NET_JOB, "speech-net", "ADR-0047 decision 1");
    assert_eq!(*NET_MAX_S.start(), u16::from(*PIECE_S.start()));
    assert_eq!(
        *NET_MAX_S.end(),
        600,
        "the controller's ruling: ten minutes"
    );
    let text = |s: &u16| MAX_TEXT_BYTES * usize::from(s / u16::from(*PIECE_S.end()));
    assert_eq!(MAX_NET_TEXT_BYTES, text(NET_MAX_S.end()));
    assert_eq!(MAX_NET_RUN_TEXT_BYTES, text(RECORD_MAX_S.end()));
    assert_eq!(MAX_NET_TEXT_BYTES, 320_000);
    assert_eq!(MAX_NET_REPLY_BYTES, 4 << 20);
    const { assert!(MAX_NET_REPLY_BYTES >= MAX_NET_TEXT_BYTES) };
    const { assert!(MAX_NET_RUN_TEXT_BYTES >= MAX_NET_TEXT_BYTES) };
    assert_eq!(
        NET_RENEW_S,
        1..=31 * DAY_S,
        "a month, the longest allowance a service names"
    );
}

#[test]
fn a_header_the_request_sets_or_the_keys_own_is_taken_in_any_case() {
    let key = "X-Test-Key";
    let taken = [
        AUTHORIZATION,
        "content-type",
        "CONTENT-LENGTH",
        "Host",
        "user-AGENT",
        key,
        "x-test-key",
    ];
    for name in taken {
        assert_eq!(unset_header(name, key), Err(BadField), "{name}");
    }
    assert_eq!(unset_header("X-AAI-Model", key), Ok(()));
    assert_eq!(unset_header("X-AAI-Model", AUTHORIZATION), Ok(()));
    assert_eq!(unset_header("authorization", AUTHORIZATION), Err(BadField));
    assert_eq!(
        (AUTHORIZATION, CONTENT_TYPE),
        ("Authorization", "Content-Type")
    );
}
