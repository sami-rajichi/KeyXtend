//! An ask's and a test's ranges: each shape's fields, the key's header, the key, the model and language codes, the
//! row's limits, the waits and the pieces, each refused by its name, and how big a piece may be.

use super::*;

/// `ask` as `check` refuses it, by the name of its bad value.
fn refused(ask: &CloudAsk) -> Option<&'static str> {
    match ask.check() {
        Err(CloudFault::Range(name)) => Some(name),
        Err(other) => panic!("not a range: {other:?}"),
        Ok(()) => None,
    }
}

/// The upload form naming its language `lang`, as a list when `list`, with `fixed` fields.
fn form_as(lang: &str, list: bool, fixed: Vec<FormField>) -> CloudShape {
    CloudShape::Form(FormFields {
        lang: lang.into(),
        list,
        fixed,
    })
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
            CloudShape::Azure,
            KeyHeader::Named("Ocp-Apim-Subscription-Key".into()),
        ),
        (CloudShape::Eleven, KeyHeader::Named("xi-api-key".into())),
        (
            CloudShape::Gemini,
            KeyHeader::Named("x-goog-api-key".into()),
        ),
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
    assert_eq!(
        ask.check(),
        Ok(()),
        "Lemonfox and Azure's classic take no model"
    );
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
fn a_model_or_language_code_that_is_no_plain_tag_is_refused() {
    let past = "a".repeat(*crate::ports::TAG_LEN.end() + 1);
    for code in ["", "fr FR", "عربي", past.as_str()] {
        let model = CloudAsk {
            model: Some(code.into()),
            ..ask()
        };
        assert_eq!(refused(&model), Some("model"), "{code:?}");
        let lang = CloudAsk {
            lang: code.into(),
            ..ask()
        };
        assert_eq!(refused(&lang), Some("lang"), "{code:?}");
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

/// The row's limits with its fewest seconds a request, which cap a piece before its megabytes do.
fn short() -> CloudLimits {
    CloudLimits {
        max_s: *CloudLimits::MAX_S.start(),
        ..limits()
    }
}

#[test]
fn a_piece_may_hold_whole_samples_within_the_rows_size_and_seconds() {
    let tight = CloudLimits {
        max_mb: 1,
        ..limits()
    };
    assert_eq!(tight.piece_bytes(), 1 << 20, "a megabyte of whole samples");
    let by_time = usize::from(short().max_s) * crate::ports::SOUND_HZ * crate::ports::SAMPLE_BYTES;
    assert_eq!(short().piece_bytes(), by_time);
}

#[test]
fn pieces_past_the_cap_or_count_or_of_half_samples_are_refused() {
    let (short, most) = (short(), short().piece_bytes());
    let pieces = |sizes: &[usize]| sizes.iter().map(|n| sound(&vec![1; *n])).collect();
    let over = CloudAsk::PIECES.end() + 1;
    let bad = [
        Vec::new(),
        pieces(&[0]),
        pieces(&[3]),
        pieces(&[2, most + 2]),
        pieces(&vec![2; over]),
    ];
    for pieces in bad {
        let count = pieces.len();
        let ask = CloudAsk {
            limits: short,
            pieces,
            ..ask()
        };
        assert_eq!(refused(&ask), Some("pieces"), "{count} pieces");
    }
    let ask = CloudAsk {
        limits: short,
        pieces: pieces(&[most, 2]),
        ..ask()
    };
    assert_eq!(ask.check(), Ok(()), "a piece at the cap fits");
}
