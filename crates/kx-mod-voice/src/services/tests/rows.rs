//! The shipped upload-form rows as the services' pages read 2026-11-17 give them (research file `[X]` lines): the
//! offered ones with Groq first, each row's address, key test, form and paywall, its models with the code each wants
//! for English, French and Arabic, and its prices for light and heavy use; a row not offered is tested all the same.

use super::*;
use kx_module_api::ports::{Fact, KeyHeader};

/// What a shipped form row sends: its address and key test (`None` for none, or typed), whether a 403 is its paywall,
/// its models, and whether it has an Other model line.
struct Want {
    id: &'static str,
    address: Option<&'static str>,
    test: Option<&'static str>,
    paywall: bool,
    models: &'static [&'static str],
    other: bool,
}

/// `OpenRouter`'s and `DeepInfra`'s Whisper models, by the ids they share.
const TURBO: &str = "openai/whisper-large-v3-turbo";
const LARGE: &str = "openai/whisper-large-v3";

/// The form rows in the table's order, offered or not (design "Connections and services").
const WANTS: [Want; 8] = [
    // [G1][G3]
    Want {
        id: "groq",
        address: Some("https://api.groq.com/openai/v1/audio/transcriptions"),
        test: Some("https://api.groq.com/openai/v1/models"),
        paywall: false,
        models: &["whisper-large-v3-turbo", "whisper-large-v3"],
        other: true,
    },
    // [O2][O3]: gpt-transcribe only, as the other models shut down on 2027-02-26.
    Want {
        id: "openai",
        address: Some("https://api.openai.com/v1/audio/transcriptions"),
        test: Some("https://api.openai.com/v1/models"),
        paywall: false,
        models: &["gpt-transcribe"],
        other: true,
    },
    // [M1][M2][M7][M15]
    Want {
        id: "mistral",
        address: Some("https://api.mistral.ai/v1/audio/transcriptions"),
        test: Some("https://api.mistral.ai/v1/models"),
        paywall: false,
        models: &[
            "voxtral-mini-transcribe-3-0",
            "voxtral-small-transcribe-3-0",
        ],
        other: true,
    },
    // [T2][T5][T7]: a 403 is a missing payment method or a plan limit.
    Want {
        id: "together",
        address: Some("https://api.together.ai/v1/audio/transcriptions"),
        test: Some("https://api.together.ai/v1/models"),
        paywall: true,
        models: &[LARGE],
        other: true,
    },
    // [L2][L3]: Whisper large-v3 with no model field, and no key test.
    Want {
        id: "lemonfox",
        address: Some("https://api.lemonfox.ai/v1/audio/transcriptions"),
        test: None,
        paywall: false,
        models: &["whisper-large-v3"],
        other: false,
    },
    Want {
        id: "custom",
        address: None,
        test: None,
        paywall: false,
        models: &[],
        other: true,
    },
    // [R1][R2][R3][R7]: a 403 is a key's spending cap.
    Want {
        id: "openrouter",
        address: Some("https://openrouter.ai/api/v1/audio/transcriptions"),
        test: Some("https://openrouter.ai/api/v1/key"),
        paywall: true,
        models: &[
            TURBO,
            LARGE,
            "openai/gpt-transcribe",
            "microsoft/mai-transcribe-2",
            "mistralai/voxtral-mini-transcribe-3-0",
        ],
        other: true,
    },
    // [D1][D3]: no key test was found on its pages.
    Want {
        id: "deepinfra",
        address: Some("https://api.deepinfra.com/v1/audio/transcriptions"),
        test: None,
        paywall: false,
        models: &[
            TURBO,
            LARGE,
            "nvidia/NVIDIA-Nemotron-3.5-Transcribe-30B-A4B",
        ],
        other: true,
    },
];

/// Text `url` as written, if any.
fn shown(url: Option<HttpsUrl>) -> Option<String> {
    url.map(|u| u.expose().to_owned())
}

#[test]
fn only_the_rows_the_owner_tests_by_hand_are_offered_groq_first_and_the_rest_stay_built() {
    let ids: Vec<&str> = shipped().offered().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, OFFERED, "ADR-0050; tasks 18 and 19 add their rows");
    let all: Vec<&str> = shipped().rows().iter().map(|r| r.id.as_str()).collect();
    let want: Vec<&str> = WANTS.iter().map(|w| w.id).collect();
    assert_eq!(all, want, "every form row stays in the table, in order");
    for id in NOT_OFFERED {
        let row = shipped().get(id).unwrap();
        assert!(!row.offered && row.untested, "{id}: hidden until tested");
    }
}

#[test]
fn each_form_row_posts_to_its_own_address_and_tests_its_key_there() {
    for w in &WANTS {
        let row = shipped().get(w.id).unwrap();
        assert_eq!(shown(row.url("")).as_deref(), w.address, "{}", w.id);
        assert_eq!(shown(row.test_url("")).as_deref(), w.test, "{}", w.id);
        assert_eq!(row.header, KeyHeader::Bearer, "{}", w.id);
        let CloudShape::Form(form) = &row.shape else {
            panic!("{}: a form", w.id)
        };
        let fields = (form.lang.as_str(), form.list, form.paywall);
        assert_eq!(fields, ("language", false, w.paywall), "{}", w.id);
        let models: Vec<&str> = row.models.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(
            (models, row.other),
            (w.models.to_vec(), w.other),
            "{}",
            w.id
        );
        assert!(row.untested, "{}: task 24 checks it with a real key", w.id);
    }
}

/// `OpenAI`'s error code for a country or region it does not serve [O5].
const REGION: &str = "unsupported_country_region_territory";

#[test]
fn only_openais_forms_read_its_country_refusal_code_as_the_region() {
    for w in &WANTS {
        let row = shipped().get(w.id).unwrap();
        let want = (w.id == "openai").then_some(REGION);
        for model in row.models.iter().map(Some).chain([None]) {
            let CloudShape::Form(form) = row.shape_for(model, false) else {
                panic!("{}: a form", w.id)
            };
            assert_eq!(form.region.as_deref(), want, "{}", w.id);
        }
    }
}

#[test]
fn openais_gpt_transcribe_sends_its_language_as_a_languages_list() {
    let row = shipped().get("openai").unwrap();
    let list = |shape: CloudShape| match shape {
        CloudShape::Form(f) => (f.lang, f.list),
        other => panic!("a form, not {other:?}"),
    };
    let gpt = row.model("gpt-transcribe");
    assert_eq!(list(row.shape_for(gpt, false)), ("languages".into(), true));
    let typed = list(row.shape_for(None, false));
    assert_eq!(
        typed,
        ("language".into(), false),
        "an Other model takes the row's"
    );
}

/// Each row's model and the code it wants for English, French and Arabic; `None` where its page names no such
/// language yet.
type Codes = (&'static str, &'static str, [Option<&'static str>; 3]);

/// The codes the pages give: ISO 639-1 for most [G1][O2][M3][T2][R1][D4], Lemonfox an English name [L2].
const CODES: [Codes; 10] = [
    (
        "groq",
        "whisper-large-v3-turbo",
        [Some("en"), Some("fr"), Some("ar")],
    ),
    (
        "openai",
        "gpt-transcribe",
        [Some("en"), Some("fr"), Some("ar")],
    ),
    (
        "mistral",
        "voxtral-small-transcribe-3-0",
        [Some("en"), Some("fr"), Some("ar")],
    ),
    ("together", LARGE, [Some("en"), Some("fr"), Some("ar")]),
    (
        "lemonfox",
        "whisper-large-v3",
        [Some("english"), Some("french"), Some("arabic")],
    ),
    ("openrouter", TURBO, [Some("en"), Some("fr"), Some("ar")]),
    (
        "openrouter",
        "microsoft/mai-transcribe-2",
        [Some("en"), Some("fr"), Some("ar")],
    ),
    (
        "openrouter",
        "mistralai/voxtral-mini-transcribe-3-0",
        [Some("en"), Some("fr"), Some("ar")],
    ),
    ("deepinfra", LARGE, [Some("en"), Some("fr"), Some("ar")]),
    (
        "deepinfra",
        "nvidia/NVIDIA-Nemotron-3.5-Transcribe-30B-A4B",
        [Some("en"), None, None],
    ),
];

#[test]
fn each_model_wants_the_code_its_page_gives_for_english_french_and_arabic() {
    for (id, model, want) in CODES {
        let m = shipped().get(id).unwrap().model(model).unwrap();
        let got = ["en", "fr", "ar"].map(|l| m.code(l));
        assert_eq!(got, want, "{id} {model}");
    }
    let lemonfox = shipped().get("lemonfox").unwrap();
    assert!(
        lemonfox.models.iter().all(|m| m.unnamed),
        "Lemonfox names no model"
    );
    let named = shipped().rows().iter().filter(|r| r.id != "lemonfox");
    assert!(named.flat_map(|r| &r.models).all(|m| !m.unnamed));
}

/// A price fact's id and its amounts in hundredths, by name: a fee in hundredths of a percent.
fn amounts(fact: &Fact) -> (&str, Vec<(&str, u32)>) {
    let value = |v: &FactValue| match v {
        FactValue::Money { amount, .. } | FactValue::Number(amount) => amount.0,
        other => panic!("a price, not {other:?}"),
    };
    let args = fact.args.iter().map(|a| (a.name.as_str(), value(&a.value)));
    (fact.id.as_str(), args.collect())
}

/// Each named row's light and heavy prices: the cheapest and dearest model, or its one price [O1][M5][T1][L1][R3][D1].
const PRICES: [(&str, &str, [u32; 2], [u32; 2]); 6] = [
    ("openai", "plain", [405, 405], [1620, 1620]),
    ("mistral", "range", [360, 315], [1440, 1260]),
    ("together", "plain", [135, 135], [540, 540]),
    ("lemonfox", "plain", [500, 500], [1000, 1000]),
    ("openrouter", "range-fee", [405, 16], [1620, 65]),
    ("deepinfra", "range", [180, 18], [720, 72]),
];

#[test]
fn each_rows_prices_are_its_pages_numbers_for_light_and_heavy_use() {
    for (id, kind, light, heavy) in PRICES {
        let facts = &shipped().get(id).unwrap().facts;
        for (fact, [high, low]) in [(&facts.light, light), (&facts.heavy, heavy)] {
            let (got, args) = amounts(fact);
            let money: Vec<(&str, u32)> = match kind {
                "plain" => vec![("price", high)],
                _ => vec![("high", high), ("low", low)],
            };
            let fee = (kind == "range-fee").then_some(("fee", 550));
            let want: Vec<(&str, u32)> = fee.into_iter().chain(money).collect();
            assert_eq!((got, args), (kind, want), "{id}");
        }
    }
}

#[test]
fn openrouters_piece_lasts_its_60_second_upstream_timeout() {
    // [R1]: "Upstream timeout is 60 s per request", so a longer recording goes in pieces.
    assert_eq!(shipped().get("openrouter").unwrap().limits.max_s, 60);
}
