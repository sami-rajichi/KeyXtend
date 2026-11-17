//! The checks a services table shares with the frames: a model's id, a language code, a key, a shape, a header and an
//! address's host, each as an ask's `check` and the frames take them.

use super::*;
use crate::ports::TAG_LEN;

/// A key made at run time from every visible ASCII byte, so no file holds one.
fn made_key() -> String {
    (b'!'..=b'~').map(char::from).collect()
}

#[test]
fn a_model_id_and_a_language_code_fit_as_an_ask_takes_them() {
    let long = "m".repeat(*TAG_LEN.end() + 1);
    for id in [MODEL, "openai/whisper-large-v3", "llama:7b", "v1.5"] {
        assert!(CloudAsk::model_fits(id), "{id}");
    }
    for id in ["", "a//b", "../x", "a b", "/x", "é", long.as_str()] {
        assert!(!CloudAsk::model_fits(id), "{id:?}");
    }
    for code in ["fr", "ar-SA", "french", "fr_FR"] {
        assert!(CloudAsk::lang_fits(code), "{code}");
    }
    for code in ["", "fr FR", "fr/FR", "ع", long.as_str()] {
        assert!(!CloudAsk::lang_fits(code), "{code:?}");
    }
    let mut odd = ask();
    odd.model = Some(Redacted::new("a//b".into()));
    assert_eq!(refused(&odd), Some("model"), "the same rule as the check");
}

#[test]
fn a_key_fits_when_it_is_visible_ascii_within_its_cap() {
    let key = made_key();
    assert!(CloudAsk::key_fits(&key));
    assert!(CloudAsk::key_fits(&"k".repeat(MAX_KEY_BYTES)));
    let bad = [
        String::new(),
        format!("{key} "),
        format!("{key}\n"),
        "k".repeat(MAX_KEY_BYTES + 1),
    ];
    for text in bad {
        assert!(!CloudAsk::key_fits(&text), "{} bytes", text.len());
    }
    let mut odd = ask();
    odd.key = secret("");
    assert_eq!(refused(&odd), Some("key"), "the same rule as the check");
}

#[test]
fn a_shape_and_a_header_say_whether_they_fit_and_need_a_model() {
    let named = KeyHeader::Named("xi-api-key".into());
    assert!(form().fits() && KeyHeader::Bearer.fits() && named.fits());
    assert!(!KeyHeader::Named("x y".into()).fits());
    let spaced = CloudShape::Form(FormFields {
        lang: "lang uage".into(),
        list: false,
        fixed: Vec::new(),
    });
    assert!(!spaced.fits());
    assert!(!form().needs_model() && !CloudShape::Azure.needs_model());
    assert!(CloudShape::Eleven.needs_model() && CloudShape::Gemini.needs_model());
}

#[test]
fn an_address_is_on_its_own_host_or_under_a_dot_entry() {
    let token = "https://kx-test.cognitiveservices.azure.com/sts/v1.0/issueToken";
    let url = HttpsUrl::new(token).unwrap();
    for entry in [
        ".cognitiveservices.azure.com",
        "kx-test.cognitiveservices.azure.com",
    ] {
        assert!(url.is_on(entry), "{entry}");
    }
    for entry in ["cognitiveservices.azure.com", ".azure.co", ".com", "", "."] {
        assert!(!url.is_on(entry), "{entry:?}");
    }
    let look = HttpsUrl::new("https://cognitiveservices.azure.com.evil.example/x").unwrap();
    assert!(!look.is_on(".cognitiveservices.azure.com"), "a look-alike");
}
