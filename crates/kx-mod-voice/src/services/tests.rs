//! The services table: the shipped rows load Groq first and each is valid, a bad row is refused with its rule, a
//! request takes the right form, host and limits, and nothing in the table can name a default.

use super::*;
use kx_module_api::ports::{
    CloudLimits, CloudShape, Date, FactValue, HttpsUrl, Hundredths, LINK_SCHEME,
};

mod parts;
mod refuse;
mod request;
mod room;

/// The compiled-in text.
const TEXT: &str = include_str!("../../services.toml");

/// An `OpenAI` row as the 2026-11-17 pages give it: `gpt-transcribe` takes `languages[]`, `whisper-1` `language`.
const OPENAI: &str = r#"
[[services]]
id = "openai-test"
name = "OpenAI"
shape = "form"
form = { lang = "language", list = false, fixed = [] }
host = "api.openai.com"
address = "https://api.openai.com/v1/audio/transcriptions"
test = "https://api.openai.com/v1/models"
header = "bearer"
keys = "https://platform.openai.com/api-keys"
limits = { cap_mb = 25, max_mb = 23, max_s = 600, per_min = 50, at_once = 2 }
terms = 1
offered = true
untested = true

[[services.models]]
id = "gpt-transcribe"
name = "GPT Transcribe"
langs = { ar = "ar", en = "en", fr = "fr" }
form = { lang = "languages", list = true }

[[services.models]]
id = "whisper-1"
name = "Whisper"
langs = { ar = "ar", en = "en", fr = "fr" }

[services.facts]
read = 2026-11-17
source = "https://developers.openai.com/api/docs/pricing"
currency = "USD"
place = { id = "us" }
trains = { id = "opt-in" }
keeps = { id = "none" }
free = { id = "none" }
light = { id = "price", money = { price = 4.05 } }
heavy = { id = "price", money = { price = 16.20 } }
"#;

/// An Azure row: a typed address under its domain, a test path, and a classic model that the request names not.
const AZURE: &str = r#"
[[services]]
id = "azure-test"
name = "Azure"
shape = "azure"
host = ".cognitiveservices.azure.com"
address = "typed"
test = "/sts/v1.0/issueToken"
header = "Ocp-Apim-Subscription-Key"
keys = "https://portal.azure.com"
limits = { cap_mb = 500, cap_s = 18000, max_mb = 23, max_s = 600, per_min = 600, at_once = 4 }
terms = 1
offered = true
untested = true

[[services.models]]
id = "classic"
name = "Azure Speech"
langs = { ar = "ar-SA", en = "en-US", fr = "fr-FR" }
unnamed = true

[[services.models]]
id = "MAI-Transcribe-2"
name = "MAI-Transcribe-2"
langs = { ar = "ar", en = "en", fr = "fr" }

[services.facts]
read = 2026-11-17
source = "https://learn.microsoft.com/en-us/azure/ai-services/speech-service/fast-transcription-create"
place = { id = "region" }
trains = { id = "unknown" }
keeps = { id = "none" }
free = { id = "none" }
light = { id = "unknown" }
heavy = { id = "unknown" }
"#;

/// An `OpenRouter` row with its keep-nothing field, and a model whose every host keeps the voice.
const ROUTER: &str = r#"
[[services]]
id = "router-test"
name = "OpenRouter"
shape = "form"
form = { lang = "language", list = false }
keep = { name = "provider", value = '{"zdr":true}' }
host = "openrouter.ai"
address = "https://openrouter.ai/api/v1/audio/transcriptions"
test = "https://openrouter.ai/api/v1/key"
header = "bearer"
keys = "https://openrouter.ai/settings/keys"
limits = { cap_mb = 25, max_mb = 23, max_s = 600, per_min = 60, at_once = 4 }
terms = 1
offered = true
untested = true

[[services.models]]
id = "openai/whisper-large-v3-turbo"
name = "Whisper Large v3 Turbo"
langs = { ar = "ar", en = "en", fr = "fr" }

[[services.models]]
id = "kept/model"
name = "Kept"
langs = { en = "en" }
keeps = true

[services.facts]
read = 2026-11-17
source = "https://openrouter.ai/docs/guides/features/zdr"
place = { id = "host" }
trains = { id = "host" }
keeps = { id = "host" }
free = { id = "small" }
light = { id = "unknown" }
heavy = { id = "unknown" }
"#;

/// The compiled-in table, which must load.
fn shipped() -> &'static Services {
    table().unwrap()
}

/// The shipped text with `rows` after it.
fn with_rows(rows: &str) -> Services {
    Services::parse(&format!("{TEXT}{rows}")).unwrap()
}

/// The text with each pair's first `from` replaced by its `to`, in turn.
fn edits_of(text: &str, pairs: &[(&str, &str)]) -> Result<Services, ServicesError> {
    let text = pairs.iter().fold(text.to_owned(), |text, (from, to)| {
        assert!(text.contains(from), "{from}");
        text.replacen(from, to, 1)
    });
    Services::parse(&text)
}

/// The shipped table with the edits.
fn edits(pairs: &[(&str, &str)]) -> Result<Services, ServicesError> {
    edits_of(TEXT, pairs)
}

/// The row error one edit of the shipped text gives.
fn row_err(from: &str, to: &str) -> Option<ServiceError> {
    match edits(&[(from, to)]) {
        Err(ServicesError::Row { why, .. }) => Some(why),
        _ => None,
    }
}

#[test]
fn the_shipped_table_loads_groq_first_with_custom() {
    let ids: Vec<&str> = shipped().rows().iter().map(|r| r.id.as_str()).collect();
    assert!(ids.contains(&"custom"), "{ids:?}");
    let first = shipped().offered().next().map(|r| r.id.as_str());
    assert_eq!(first, Some("groq"), "edge 24: Groq is listed first");
    assert_eq!((shipped().light_hours(), shipped().heavy_hours()), (15, 60));
    assert_eq!(Services::parse(TEXT).as_ref(), Ok(shipped()));
}

#[test]
fn every_shipped_row_is_https_with_fitting_limits_and_only_custom_is_unnamed() {
    let unnamed: Vec<&str> = shipped()
        .rows()
        .iter()
        .filter(|r| r.name.is_none())
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(unnamed, ["custom"], "one unnamed row, the Custom one");
    for row in shipped().rows() {
        let id = &row.id;
        let named = row.name.is_some();
        match &row.address {
            Address::Fixed(url) => assert!(url.expose().starts_with(LINK_SCHEME), "{id}"),
            Address::Typed { under } => assert_eq!(under.is_some(), named, "{id}"),
        }
        assert!(row.shape.fits() && row.terms >= 1, "{id}");
        assert!(row.other || !row.models.is_empty(), "{id}");
        assert!(row.models.iter().all(|m| !m.langs.is_empty()), "{id}");
        let l = row.limits;
        assert!(CloudLimits::MAX_MB.contains(&l.max_mb) && CloudLimits::MAX_S.contains(&l.max_s));
        assert!(
            CloudLimits::PER_MIN.contains(&l.per_min) && CloudLimits::AT_ONCE.contains(&l.at_once)
        );
        let linked = (row.keys.is_some(), row.facts.source.is_some());
        assert_eq!(
            linked,
            (named, named),
            "{id}: a named row links its keys and facts pages"
        );
    }
}

#[test]
fn groqs_facts_are_numbers_and_flags_with_their_day_and_source() {
    let groq = shipped().get("groq").unwrap();
    let facts = &groq.facts;
    assert_eq!(facts.read, Date::new(2026, 11, 17).unwrap());
    let spoken = groq
        .models
        .iter()
        .all(|m| ["en", "fr", "ar"].iter().all(|l| m.code(l).is_some()));
    assert!(spoken, "Groq's models know the owner's three languages");
    assert_eq!(
        facts.source.as_ref().map(HttpsUrl::host),
        Some("console.groq.com")
    );
    assert_eq!(
        (facts.place.id.as_str(), facts.trains.id.as_str()),
        ("us", "never")
    );
    assert!(
        facts.warn.is_none() && !facts.trains.warn,
        "Groq trains on nothing"
    );
    let free = &facts.free.args;
    let counts: Vec<(&str, &FactValue)> =
        free.iter().map(|a| (a.name.as_str(), &a.value)).collect();
    assert_eq!(
        counts,
        [
            ("hours", &FactValue::Count(8)),
            ("per_min", &FactValue::Count(20))
        ]
    );
    let usd = |amount| FactValue::Money {
        amount: Hundredths(amount),
        currency: "USD".into(),
    };
    let heavy: Vec<&FactValue> = facts.heavy.args.iter().map(|a| &a.value).collect();
    assert_eq!(heavy, [&usd(666), &usd(240)], "high then low, by name");
}

#[test]
fn no_row_can_name_a_default_or_a_field_the_table_does_not_know() {
    for extra in ["default = true", "lead = true", "pick = \"groq\""] {
        let to = format!("id = \"groq\"\n{extra}");
        let refused = edits(&[("id = \"groq\"", &to)]);
        assert!(matches!(refused, Err(ServicesError::Parse(_))), "{extra}");
    }
}

#[test]
fn a_bad_table_turns_cloud_off_and_never_panics() {
    for text in ["", "light_hours = 15", "services = []", "\u{0}"] {
        assert!(Services::parse(text).is_err(), "{text:?}");
    }
}
