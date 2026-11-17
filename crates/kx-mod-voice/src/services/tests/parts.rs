//! The validator refuses a broken model by its id and a broken fact by its name, with the rule in plain words.

use super::*;
use FactError as F;
use ModelError as M;

/// Groq's light price and free use, which the fact edits below break.
const LIGHT: &str = "light = { id = \"free-or\", money = { low = 0.60, high = 1.67 } }";
const FREE: &str = "free = { id = \"daily\", count = { hours = 8, per_min = 20 } }";

/// The model error one edit of the shipped text with `rows` after it gives, with the model's id.
fn model_err(rows: &str, from: &str, to: &str) -> Option<(String, ModelError)> {
    match edits_of(&format!("{TEXT}{rows}"), &[(from, to)]) {
        Err(ServicesError::Row {
            why: ServiceError::Model { id, why },
            ..
        }) => Some((id, why)),
        _ => None,
    }
}

/// Asserts that one edit of the shipped table, `from` to `to`, refuses the fact `name` with `why`.
#[track_caller]
fn refused(from: &str, to: &str, name: &'static str, why: FactError) {
    assert_eq!(
        row_err(from, to),
        Some(ServiceError::Fact(name, why)),
        "{to}"
    );
}

#[test]
fn a_model_that_breaks_a_rule_is_refused_by_its_id() {
    let gpt = |why| Some(("gpt-transcribe".to_owned(), why));
    let quoted = "\"gpt-transcribe\"";
    assert_eq!(
        model_err(OPENAI, quoted, "\"a//b\""),
        Some(("a//b".into(), M::Id))
    );
    let twice = Some(("whisper-1".to_owned(), M::Twice));
    assert_eq!(model_err(OPENAI, quoted, "\"whisper-1\""), twice);
    assert_eq!(
        model_err(OPENAI, "\"GPT Transcribe\"", "\"\""),
        gpt(M::Name)
    );
    let langs = "langs = { ar = \"ar\", en = \"en\", fr = \"fr\" }";
    for bad in [
        "{}",
        "{ \"fr-FR\" = \"fr\" }",
        "{ ar = \"ع\" }",
        "{ ar = \"a r\" }",
    ] {
        let to = format!("langs = {bad}");
        assert_eq!(model_err(OPENAI, langs, &to), gpt(M::Langs), "{bad}");
    }
    let own = "form = { lang = \"languages\", list = true }";
    let odd = own.replace("\"languages\"", "\"a b\"");
    assert_eq!(model_err(OPENAI, own, &odd), gpt(M::Form));
    assert_eq!(
        model_err(OPENAI, own, &format!("{own}\nkeeps = true")),
        gpt(M::Keeps)
    );
}

#[test]
fn a_model_takes_a_named_language_set_that_must_exist() {
    let groq = shipped().get("groq").unwrap();
    let whisper = &groq.models[0].langs;
    assert!(
        groq.models.iter().all(|m| m.langs == *whisper),
        "one set, written once"
    );
    let codes = ["en", "fr", "ar", "fil"].map(|l| whisper.get(l).map(String::as_str));
    assert_eq!(codes, [Some("en"), Some("fr"), Some("ar"), Some("tl")]);
    let turbo = Some(("whisper-large-v3-turbo".to_owned(), M::Set));
    let typo = edits(&[("langs = \"whisper\"", "langs = \"whispr\"")]);
    let typo = match typo {
        Err(ServicesError::Row {
            why: ServiceError::Model { id, why },
            ..
        }) => Some((id, why)),
        _ => None,
    };
    assert_eq!(typo, turbo, "a set the table does not name");
}

#[test]
fn a_models_own_form_that_asks_for_timings_says_so() {
    let own = "form = { lang = \"languages\", list = true }";
    let timed = "form = { lang = \"languages\", list = true, fixed = [{ name = \"timestamp_granularities\", value = \"word\" }] }";
    let err = model_err(OPENAI, own, timed);
    assert_eq!(err, Some(("gpt-transcribe".into(), M::Reply)));
    let said = err.map(|(_, why)| why.to_string()).unwrap_or_default();
    assert!(said.contains("timings"), "{said}");
}

#[test]
fn groqs_least_billed_length_is_a_fact_the_validator_checks() {
    let bills = shipped().get("groq").unwrap().facts.bills.clone().unwrap();
    assert_eq!((bills.id.as_str(), bills.args.len()), ("least", 1));
    assert!(shipped().get("custom").unwrap().facts.bills.is_none());
    let least = "bills = { id = \"least\", count = { seconds = 10 } }";
    refused(least, &least.replace("least", "Least"), "bills", F::Id);
}

#[test]
fn a_model_on_a_shape_that_needs_a_name_or_takes_no_form_is_refused() {
    let classic = |why| Some(("classic".to_owned(), why));
    let eleven = model_err(AZURE, "shape = \"azure\"", "shape = \"eleven\"");
    assert_eq!(eleven, classic(M::Unnamed));
    let form = "\nunnamed = true\nform = { lang = \"language\", list = false }";
    assert_eq!(model_err(AZURE, "\nunnamed = true", form), classic(M::Form));
}

#[test]
fn a_fact_that_breaks_a_rule_is_refused_by_its_name() {
    let source = "source = \"https://console.groq.com/docs/your-data\"\n";
    refused(
        "read = 2026-11-17",
        "read = 2026-11-17T10:00:00",
        "facts",
        F::Day,
    );
    refused(
        source,
        &source.replace("https://", "http://"),
        "facts",
        F::Source,
    );
    refused(source, "", "facts", F::Source);
    refused(
        "currency = \"USD\"",
        "currency = \"usd\"",
        "facts",
        F::Currency,
    );
    refused("currency = \"USD\"\n", "", "light", F::Currency);
    refused(LIGHT, &LIGHT.replace("0.60", "0.605"), "light", F::Amount);
    refused(LIGHT, &LIGHT.replace("0.60", "-1.0"), "light", F::Amount);
    refused(
        "place = { id = \"us\" }",
        "place = { id = \"US\" }",
        "place",
        F::Id,
    );
    let switch = "switch = \"Settings → Data Controls\"";
    refused(switch, "switch = \"a\\nb\"", "keeps", F::Text);
}

#[test]
fn a_fact_value_with_an_odd_or_twice_given_name_or_day_is_refused() {
    let twice = FREE.replace("per_min = 20 }", "per_min = 20 }, number = { hours = 1.0 }");
    refused(FREE, &twice, "free", F::Arg);
    refused(FREE, &FREE.replace("hours", "Hours"), "free", F::Arg);
    let day = "free = { id = \"daily\", day = { until = 2026-12-31T00:00:00 } }";
    refused(FREE, day, "free", F::Day);
}

#[test]
fn a_model_refusal_names_its_rule_in_plain_words() {
    let err =
        model_err(OPENAI, "\"GPT Transcribe\"", "\"\"").map(|(id, why)| format!("{id}: {why}"));
    let want = "gpt-transcribe: its name is empty or holds a hidden mark";
    assert_eq!(err.as_deref(), Some(want));
}
