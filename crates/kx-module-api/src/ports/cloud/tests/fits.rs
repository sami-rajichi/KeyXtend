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
        paywall: false,
        region: None,
    });
    assert!(!spaced.fits());
    let long = "r".repeat(*TAG_LEN.end() + 1);
    for code in ["unsupported country", "", long.as_str()] {
        let CloudShape::Form(mut odd) = form() else {
            unreachable!("a form")
        };
        odd.region = Some(code.to_owned());
        assert!(!CloudShape::Form(odd).fits(), "{code:?}");
    }
    assert!(!form().needs_model());
    assert!(eleven().needs_model() && CloudShape::Gemini.needs_model());
    assert!(assembly().needs_model() && assembly().fits());
    for name in ["X AAI-Model", "X-AAI:Model", "", long.as_str()] {
        let odd = CloudShape::Assembly {
            model_header: name.to_owned(),
        };
        assert!(!odd.fits(), "a model header of {name:?}");
    }
}

#[test]
fn a_model_header_never_names_a_header_the_request_sets_or_the_keys_own() {
    let named = KeyHeader::Named("X-Test-Key".to_owned());
    assert_eq!(KeyHeader::Bearer.name(), "Authorization");
    assert!(assembly().fits_beside(&KeyHeader::Bearer) && assembly().fits_beside(&named));
    let shape = |name: &str| CloudShape::Assembly {
        model_header: name.to_owned(),
    };
    for name in [
        "authorization",
        "Content-Type",
        "content-length",
        "HOST",
        "User-AGENT",
    ] {
        assert!(!shape(name).fits_beside(&KeyHeader::Bearer), "{name}");
    }
    assert!(
        !shape("x-test-key").fits_beside(&named),
        "the key's own, in any case"
    );
    assert!(CloudShape::Gemini.fits_beside(&named) && eleven().fits_beside(&named));
}

#[test]
fn elevenlabs_own_form_holds_its_language_and_fixed_fields_and_fits_by_its_own_check() {
    assert!(eleven().fits());
    let CloudShape::Eleven(mut odd) = eleven() else {
        unreachable!("its own form")
    };
    assert!(odd.fits(), "its own check, with no shape around it");
    odd.fixed[0].name = "tag audio".into();
    let spaced = !odd.fits() && !CloudShape::Eleven(odd.clone()).fits();
    assert!(spaced, "a spaced name");
    odd.fixed[0].name = "tag_audio_events".into();
    odd.fixed[0].value = "a\r\nb".into();
    assert!(!odd.fits(), "a breaking value");
    let one = FormField {
        name: "a".into(),
        value: "1".into(),
    };
    odd.fixed = vec![one; MAX_FIXED + 1];
    assert!(!odd.fits(), "past MAX_FIXED");
    let CloudShape::Form(form) = form() else {
        unreachable!("a form")
    };
    assert!(form.fits(), "an upload form's own check is public too");
}

#[test]
fn an_address_is_on_its_own_host_or_under_a_dot_entry() {
    let test = "https://kx-test.speech.example.com/v1/models";
    let url = HttpsUrl::new(test).unwrap();
    for entry in [".speech.example.com", "kx-test.speech.example.com"] {
        assert!(url.is_on(entry), "{entry}");
    }
    for entry in ["speech.example.com", ".example.co", ".com", "", "."] {
        assert!(!url.is_on(entry), "{entry:?}");
    }
    let look = HttpsUrl::new("https://speech.example.com.evil.example/x").unwrap();
    assert!(!look.is_on(".speech.example.com"), "a look-alike");
}
