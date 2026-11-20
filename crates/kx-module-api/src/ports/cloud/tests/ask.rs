//! An ask's and a test's ranges: each shape's fields, the key's header, the key, the model and language codes, the
//! row's limits and the waits, each refused by its name; the pieces are in `pieces`.

use super::*;

/// The upload form naming its language `lang`, as a list when `list`, with `fixed` fields.
fn form_as(lang: &str, list: bool, fixed: Vec<FormField>) -> CloudShape {
    CloudShape::Form(FormFields {
        lang: lang.into(),
        list,
        fixed,
        paywall: false,
        region: None,
    })
}

/// `shape` with its 403 read as a paywall, as Together's row says.
fn paywalled(shape: CloudShape) -> CloudShape {
    match shape {
        CloudShape::Form(form) => CloudShape::Form(FormFields {
            paywall: true,
            ..form
        }),
        other => other,
    }
}

/// `shape` with `OpenAI`'s region refusal code, read as the region fault.
fn regioned(shape: CloudShape) -> CloudShape {
    match shape {
        CloudShape::Form(form) => CloudShape::Form(FormFields {
            region: Some("unsupported_country_region_territory".into()),
            ..form
        }),
        other => other,
    }
}

/// A fixed field.
fn field(name: &str, value: &str) -> FormField {
    FormField {
        name: name.into(),
        value: value.into(),
    }
}

#[test]
fn every_shape_with_its_header_passes() {
    let zdr = vec![field("provider", r#"{"zdr":true}"#)];
    let rows = [
        (form_as("languages", true, Vec::new()), KeyHeader::Bearer),
        (form_as("language", false, zdr), KeyHeader::Bearer),
        (
            paywalled(form_as("language", false, Vec::new())),
            KeyHeader::Bearer,
        ),
        (
            regioned(form_as("language", false, Vec::new())),
            KeyHeader::Bearer,
        ),
        (eleven(), KeyHeader::Named("xi-api-key".into())),
        (
            CloudShape::Gemini,
            KeyHeader::Named("x-goog-api-key".into()),
        ),
        (assembly(), KeyHeader::Bearer),
    ];
    for (shape, header) in rows {
        let ask = CloudAsk {
            shape: shape.clone(),
            header: header.clone(),
            ..ask()
        };
        assert_eq!(ask.check(), Ok(()), "{shape:?} {header:?}");
    }
    assert_eq!(test().check(), Ok(()));
}

#[test]
fn a_row_with_no_model_field_passes() {
    let ask = CloudAsk {
        model: None,
        lang: "french".into(),
        ..ask()
    };
    assert_eq!(ask.check(), Ok(()), "Lemonfox takes no model");
}

#[test]
fn a_form_that_could_break_a_request_is_refused() {
    let many = vec![field("a", "1"); MAX_FIXED + 1];
    let full = vec![field("a", "1"); MAX_FIXED];
    let long = "v".repeat(MAX_FIELD_VALUE + 1);
    let bad = [
        form_as("", false, Vec::new()),
        form_as("lang uage", false, Vec::new()),
        form_as("languages[]", true, Vec::new()),
        form_as("lang\"x", false, Vec::new()),
        form_as("language", false, many),
        form_as(
            "language",
            false,
            vec![field("provider", "{\"zdr\":\r\ntrue}")],
        ),
        form_as("language", false, vec![field("", "1")]),
        form_as("language", false, vec![field("provider", &long)]),
    ];
    for shape in bad {
        let ask = CloudAsk {
            shape: shape.clone(),
            ..ask()
        };
        assert_eq!(refused(&ask), Some("shape"), "{shape:?}");
    }
    let ask = CloudAsk {
        shape: form_as("language", false, full),
        ..ask()
    };
    assert_eq!(ask.check(), Ok(()), "MAX_FIXED fields fit");
}

#[test]
fn a_header_or_a_key_that_could_break_a_requests_head_is_refused() {
    for name in ["", "xi api key", "x-key:", "x\r\nkey"] {
        let ask = CloudAsk {
            header: KeyHeader::Named(name.into()),
            ..ask()
        };
        assert_eq!(refused(&ask), Some("header"), "{name:?}");
    }
    let long = "k".repeat(MAX_KEY_BYTES + 1);
    for key in ["", "gsk 7731", "gsk\r\nX-Other: 1", "مفتاح", long.as_str()] {
        let ask = CloudAsk {
            key: secret(key),
            ..ask()
        };
        assert_eq!(refused(&ask), Some("key"));
        let test = CloudTest {
            key: secret(key),
            ..test()
        };
        assert_eq!(test.check(), Err(CloudFault::Range("key")));
    }
}

#[test]
fn a_model_header_naming_a_header_the_request_sets_is_refused_as_the_shape() {
    let shape = |name: &str| CloudShape::Assembly {
        model_header: name.to_owned(),
    };
    let named = KeyHeader::Named("X-Test-Key".into());
    let rows = [
        (shape("authorization"), KeyHeader::Bearer),
        (shape("user-agent"), KeyHeader::Bearer),
        (shape("x-test-key"), named),
    ];
    for (shape, header) in rows {
        let ask = CloudAsk {
            shape,
            header,
            ..ask()
        };
        assert_eq!(refused(&ask), Some("shape"));
    }
}

/// `ask()` for `model` in `lang`.
fn ask_as(model: Option<&str>, lang: &str) -> CloudAsk {
    CloudAsk {
        model: model.map(|m| Redacted::new(m.into())),
        lang: lang.into(),
        ..ask()
    }
}

#[test]
fn real_model_ids_and_language_codes_pass() {
    let ids = [
        "whisper-large-v3-turbo",
        "gpt-transcribe",
        "openai/whisper-large-v3",
        "voxtral-mini-transcribe-3-0",
        "scribe_v3",
        "gemini-3.5-transcribe",
        "MAI-Transcribe-2",
    ];
    for id in ids {
        assert_eq!(ask_as(Some(id), "fr").check(), Ok(()), "{id}");
    }
    for lang in ["fr", "ar-SA", "french", "zh_Hans"] {
        assert_eq!(ask_as(Some(MODEL), lang).check(), Ok(()), "{lang}");
    }
}

#[test]
fn a_model_or_language_code_that_could_break_a_request_is_refused() {
    let past = "a".repeat(*crate::ports::TAG_LEN.end() + 1);
    let both = [
        "",
        "fr FR",
        "عربي",
        "a\"b",
        "a\\b",
        "a?b",
        "a#b",
        "a%b",
        past.as_str(),
    ];
    for code in both.iter().chain(&["/ab", "ab/", "a//b", "a/../b"]) {
        assert_eq!(
            refused(&ask_as(Some(*code), "fr")),
            Some("model"),
            "{code:?}"
        );
    }
    for code in both.iter().chain(&["fr/CA", "fr.CA", "fr:CA"]) {
        assert_eq!(
            refused(&ask_as(Some(MODEL), code)),
            Some("lang"),
            "{code:?}"
        );
    }
}

#[test]
fn elevenlabs_gemini_and_assemblyai_need_a_model_and_the_form_takes_none() {
    let rows = [
        (eleven(), Some("model")),
        (CloudShape::Gemini, Some("model")),
        (assembly(), Some("model")),
        (form(), None),
    ];
    for (shape, want) in rows {
        let ask = CloudAsk {
            shape: shape.clone(),
            ..ask_as(None, "fr")
        };
        assert_eq!(refused(&ask), want, "{shape:?}");
    }
}

/// A value's name and a change that puts it just past its range.
type Case = (&'static str, fn(&mut CloudAsk));

/// Each limit and wait just past either end of its range, and the job's.
const PAST: [Case; 13] = [
    ("max_mb", |a| a.limits.max_mb = 0),
    ("max_mb", |a| {
        a.limits.max_mb = CloudLimits::MAX_MB.end() + 1;
    }),
    ("max_s", |a| a.limits.max_s = CloudLimits::MAX_S.start() - 1),
    ("max_s", |a| a.limits.max_s = CloudLimits::MAX_S.end() + 1),
    ("per_min", |a| a.limits.per_min = 0),
    ("per_min", |a| {
        a.limits.per_min = CloudLimits::PER_MIN.end() + 1;
    }),
    ("at_once", |a| a.limits.at_once = 0),
    ("at_once", |a| {
        a.limits.at_once = CloudLimits::AT_ONCE.end() + 1;
    }),
    ("wait_s", |a| a.wait_s = CloudAsk::WAIT_S.start() - 1),
    ("wait_s", |a| a.wait_s = CloudAsk::WAIT_S.end() + 1),
    ("tries", |a| a.tries = 0),
    ("tries", |a| a.tries = CloudAsk::TRIES.end() + 1),
    ("job_mb", |a| a.job.job_mb = 0),
];

#[test]
fn the_ranges_are_the_frames_own_limits() {
    assert_eq!(MAX_KEY_BYTES, kx_ipc::MAX_KEY_BYTES);
    assert_eq!(MAX_FIXED, kx_ipc::MAX_FIXED);
    assert_eq!(CloudLimits::MAX_MB, kx_ipc::NET_MAX_MB);
    assert_eq!(CloudLimits::PER_MIN, kx_ipc::NET_PER_MIN);
    assert_eq!(CloudLimits::AT_ONCE, kx_ipc::NET_AT_ONCE);
    assert_eq!(CloudAsk::WAIT_S, kx_ipc::NET_WAIT_S);
    assert_eq!(CloudAsk::TRIES, kx_ipc::NET_TRIES);
}

#[test]
fn each_limit_and_wait_outside_its_range_is_named() {
    for (name, change) in PAST {
        let mut ask = ask();
        change(&mut ask);
        assert_eq!(refused(&ask), Some(name), "{ask:?}");
    }
    let slow = CloudTest {
        wait_s: CloudAsk::WAIT_S.end() + 1,
        ..test()
    };
    assert_eq!(slow.check(), Err(CloudFault::Range("wait_s")));
}
