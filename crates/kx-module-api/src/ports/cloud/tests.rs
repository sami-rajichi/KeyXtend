//! The cloud speech port: its gate, an address that is only ever HTTPS, and an ask, a test and a note that never print
//! the key, the sound, the address or a text. Each value's range is in `ask`.

use super::*;
use crate::ports::JobLimits;
use crate::{REDACTED_MARKER, Redacted, SecretBytes, Zeroizing};

mod ask;
mod fits;
mod pieces;
mod url;

/// The length of the key `key` makes.
const KEY_LEN: usize = 40;
/// Sound no output may show, as `Debug` would print its bytes.
const SOUND: [u8; 4] = [0x7f, 0x3e, 0x11, 0x5a];
/// The bytes of `SOUND` as a plain `Debug` prints them.
const SOUND_SHOWN: &str = "127, 62, 17, 90";
/// Groq's address, from its row.
const GROQ: &str = "https://api.groq.com/openai/v1/audio/transcriptions";
/// A model id no output may show, as a typed one is the user's.
const MODEL: &str = "whisper-large-v3-turbo";
/// A transcript no output may show.
const SAID: &str = "bonjour-4417";

/// A key no output may show, made up at run time so no file holds one.
fn key() -> String {
    let marks = ('a'..='z').rev().chain('0'..='9');
    marks.cycle().step_by(3).take(KEY_LEN).collect()
}

/// `text` as a secret.
fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// `bytes` as secret sound.
fn sound(bytes: &[u8]) -> SecretBytes {
    Redacted::new(Zeroizing::new(bytes.to_vec()))
}

/// The job's limits as the `[voice]` defaults give them.
fn job() -> JobLimits {
    JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

/// The upload form as Groq names its fields.
fn form() -> CloudShape {
    CloudShape::Form(FormFields {
        lang: "language".into(),
        list: false,
        fixed: Vec::new(),
        paywall: false,
        region: None,
    })
}

/// `ElevenLabs`' shape, its form naming its language field and asking for no sound tags.
fn eleven() -> CloudShape {
    let fixed = vec![FormField {
        name: "tag_audio_events".into(),
        value: "false".into(),
    }];
    CloudShape::Eleven(OwnFields {
        lang: "language_code".into(),
        fixed,
    })
}

/// `AssemblyAI`'s shape, naming the header its model goes in.
fn assembly() -> CloudShape {
    CloudShape::Assembly {
        model_header: "X-AAI-Model".into(),
    }
}

/// Groq's limits on its free plan.
fn limits() -> CloudLimits {
    CloudLimits {
        max_mb: 25,
        max_s: 600,
        per_min: 20,
        at_once: 2,
    }
}

/// `ask` as `check` refuses it, by the name of its bad value.
fn refused(ask: &CloudAsk) -> Option<&'static str> {
    match ask.check() {
        Err(CloudFault::Range(name)) => Some(name),
        Err(other) => panic!("not a range: {other:?}"),
        Ok(()) => None,
    }
}

/// One French piece for Groq's turbo model.
fn ask() -> CloudAsk {
    CloudAsk {
        job: job(),
        shape: form(),
        url: HttpsUrl::new(GROQ).unwrap(),
        header: KeyHeader::Bearer,
        key: secret(&key()),
        model: Some(Redacted::new(MODEL.into())),
        lang: "fr".into(),
        limits: limits(),
        wait_s: 60,
        tries: 2,
        pieces: vec![sound(&SOUND)],
    }
}

/// Groq's key test: its models list.
fn test() -> CloudTest {
    CloudTest {
        job: job(),
        shape: form(),
        url: HttpsUrl::new("https://api.groq.com/openai/v1/models").unwrap(),
        header: KeyHeader::Bearer,
        key: secret(&key()),
        wait_s: 60,
    }
}

/// Asserts `shown` holds no key, sound, address, model or text, and the marker in their place.
fn keeps_secrets(shown: &str) {
    let key = key();
    for secret in [key.as_str(), SOUND_SHOWN, "api.groq.com", MODEL, SAID] {
        assert!(!shown.contains(secret), "{shown} shows {secret}");
    }
    assert!(shown.contains(REDACTED_MARKER), "{shown}");
}

#[test]
fn the_cloud_speech_service_is_named_and_gated_by_the_network() {
    assert_eq!(CloudSpeechKey::ID.as_str(), "cloud-speech");
    let gate = Some(Capability::Network);
    assert_eq!(CloudSpeechKey::CAPABILITY, gate, "ADR-0047 decision 11");
}

#[test]
fn an_address_that_is_not_https_cannot_be_built() {
    let long = format!("{GROQ}/{}", "a".repeat(kx_ipc::MAX_URL_BYTES));
    let bad = [
        "",
        "http://api.groq.com/openai/v1/audio/transcriptions",
        "HTTPS://api.groq.com/v1",
        "ftp://api.groq.com/v1",
        "api.groq.com/openai/v1",
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
        "https://192.168.1.20:8443/v1",
        "https://2130706433/v1",
        "https://0x7f.1/v1",
        long.as_str(),
    ];
    for text in bad {
        assert_eq!(HttpsUrl::new(text), None, "{text:?}");
    }
}

#[test]
fn an_https_address_keeps_its_text_and_names_its_host() {
    let colon =
        "https://kx-test.speech.example.com/v1/transcriptions:transcribe?api-version=2025-10-15";
    let good = [
        (GROQ, "api.groq.com"),
        (colon, "kx-test.speech.example.com"),
        ("https://stt.lan:8443/v1/audio/transcriptions", "stt.lan"),
        ("https://localhost:8000", "localhost"),
        (
            "https://api.elevenlabs.io/v1/speech-to-text?x=1",
            "api.elevenlabs.io",
        ),
    ];
    for (text, host) in good {
        let url = HttpsUrl::new(text).unwrap_or_else(|| panic!("{text}"));
        assert_eq!((url.expose(), url.host()), (text, host));
    }
}

#[test]
fn an_address_prints_redacted_as_a_typed_one_is_the_users() {
    let url = HttpsUrl::new(GROQ).unwrap();
    let shown = format!("{url:?}");
    assert!(!shown.contains("groq"), "{shown}");
    assert!(shown.contains(REDACTED_MARKER), "{shown}");
}

#[test]
fn an_ask_prints_no_key_no_sound_no_address_and_no_model() {
    let ask = ask();
    let shown = format!("{ask:?}{ask:#?}");
    keeps_secrets(&shown);
    for part in ["lang: \"fr\"", "Bearer", "max_mb: 25"] {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
}

#[test]
fn a_keys_test_prints_no_key_and_no_address() {
    let test = test();
    keeps_secrets(&format!("{test:?}{test:#?}"));
}

#[test]
fn a_pieces_text_never_prints_but_its_place_and_a_fault_do() {
    let notes = [
        CloudNote::Piece {
            index: 3,
            text: secret(SAID),
        },
        CloudNote::Done,
        CloudNote::Fault(CloudFault::Allowance { renews_in: None }),
    ];
    let shown = format!("{notes:?}");
    keeps_secrets(&shown);
    for part in ["index: 3", "Done", "Allowance"] {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
}

#[test]
fn each_fault_has_its_own_line_and_none_holds_a_secret() {
    let faults = [
        CloudFault::Range("key"),
        CloudFault::Key,
        CloudFault::Credit,
        CloudFault::Account,
        CloudFault::Allowance { renews_in: None },
        CloudFault::Rate,
        CloudFault::TooBig,
        CloudFault::Language,
        CloudFault::Region,
        CloudFault::Empty,
        CloudFault::Strange,
        CloudFault::Network,
        CloudFault::Timeout,
        CloudFault::Gone,
    ];
    let lines: std::collections::HashSet<String> = faults.iter().map(ToString::to_string).collect();
    assert_eq!(lines.len(), faults.len(), "{lines:?}");
    let renews = CloudFault::Allowance {
        renews_in: Some(std::time::Duration::from_secs(90)),
    };
    assert_eq!(format!("{renews:?}"), "Allowance { renews_in: Some(90s) }");
}
