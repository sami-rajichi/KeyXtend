//! The validator refuses a broken model by its id and a broken fact by its name, with the rule in plain words.

use super::*;
use FactError as F;
use ModelError as M;

/// The model error one edit of `rows`, put after the shipped text, gives, with the model's id.
fn model_err(rows: &str, from: &str, to: &str) -> Option<(String, ModelError)> {
    let rows = edits_text(rows, &[(from, to)]);
    match parse(&format!("{TEXT}{rows}")) {
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
    let sets: BTreeMap<String, LangCodes> = toml::from_str(LANGS).unwrap();
    assert_eq!(
        *whisper, sets["whisper"],
        "the set as its own file writes it"
    );
    let turbo = Some((groq.models[0].id.clone(), M::Set));
    let typo = edits(&[(groq_line("langs"), "langs = \"no-such-set\"")]);
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
    let least = groq_line("bills");
    let bills = shipped().get("groq").unwrap().facts.bills.clone().unwrap();
    let id = field_of(least, "id").trim_matches('"');
    assert_eq!(bills.id, id, "the row's own fact, as its line writes it");
    assert!(shipped().get("custom").unwrap().facts.bills.is_none());
    let upper = with_field(least, "id", &field_of(least, "id").to_uppercase());
    refused(least, &upper, "bills", F::Id);
}

#[test]
fn a_model_on_a_shape_that_needs_a_name_or_takes_no_form_is_refused() {
    let base = |why| Some(("base".to_owned(), why));
    let shape = "shape = \"form\"";
    let eleven = edits_text(DOMAIN, &[(shape, "shape = \"eleven\"")]);
    assert_eq!(
        model_err(DOMAIN, shape, "shape = \"eleven\""),
        base(M::Unnamed)
    );
    let form = "\nform = { lang = \"language\", list = false }";
    assert_eq!(model_err(&eleven, "\nunnamed = true", form), base(M::Form));
}

#[test]
fn a_fact_that_breaks_a_rule_is_refused_by_its_name() {
    let (read, source) = (groq_line("read"), groq_line("source"));
    refused(read, &format!("{read}T10:00:00"), "facts", F::Day);
    let http = source.replace("https://", "http://");
    refused(source, &http, "facts", F::Source);
    refused(&format!("{source}\n"), "", "facts", F::Source);
    let currency = groq_line("currency");
    refused(currency, &currency.to_lowercase(), "facts", F::Currency);
    refused(&format!("{currency}\n"), "", "light", F::Currency);
    let light = groq_line("light");
    for low in ["0.605", "-1.0"] {
        refused(light, &with_field(light, "low", low), "light", F::Amount);
    }
    let place = groq_line("place");
    let upper = with_field(place, "id", &field_of(place, "id").to_uppercase());
    refused(place, &upper, "place", F::Id);
    let keeps = groq_line("keeps");
    let odd = with_field(keeps, "switch", "\"a\\nb\"");
    refused(keeps, &odd, "keeps", F::Text);
}

#[test]
fn a_fact_value_with_an_odd_or_twice_given_name_or_day_is_refused() {
    let free = groq_line("free");
    let raw: toml::Table = free.parse().unwrap();
    let mut groups = raw["free"].as_table().unwrap().iter();
    let (kind, args) = groups.find_map(|(k, v)| Some((k, v.as_table()?))).unwrap();
    let first = args.keys().next().unwrap();
    let group = field_of(free, kind);
    let (other, one) = if kind == "number" {
        ("count", "1")
    } else {
        ("number", "1.0")
    };
    let twice = with_field(
        free,
        kind,
        &format!("{group}, {other} = {{ {first} = {one} }}"),
    );
    refused(free, &twice, "free", F::Arg);
    let upper = group.replacen(first.as_str(), &first.to_uppercase(), 1);
    refused(free, &with_field(free, kind, &upper), "free", F::Arg);
    let until = "day = { until = 2026-12-31T00:00:00 }";
    let day = format!("free = {{ id = {}, {until} }}", field_of(free, "id"));
    refused(free, &day, "free", F::Day);
}

#[test]
fn a_model_refusal_names_its_rule_in_plain_words() {
    let err =
        model_err(OPENAI, "\"GPT Transcribe\"", "\"\"").map(|(id, why)| format!("{id}: {why}"));
    let want = "gpt-transcribe: its name is empty or holds a hidden mark";
    assert_eq!(err.as_deref(), Some(want));
}
