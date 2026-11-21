//! The cloud rules: no cloud model is ever a default, a service is off until the user confirms its card, and edges 11
//! (a hidden run), 15 (old facts), 16 (newer terms), 21 (`cloud` off) and 24 (Groq first).

use super::*;
use crate::models::{self, Pick};
use crate::services::table;
use crate::services::tests::{ROUTER, TEXT, parse, with_rows};
use kx_module_api::ports::{HttpsUrl, ServiceName, Unavailable};

mod assembly;
mod eleven;
mod gemini;
mod keep;
mod offered;
mod props;
mod router;
mod typed;
mod warn;

/// An address Custom takes: HTTPS on a host name.
const ADDRESS: &str = "https://stt.example.com/v1/audio/transcriptions";

/// The day the tests confirm on.
fn today() -> Date {
    Date::new(2026, 11, 17).unwrap()
}

/// `today` moved by `days`.
fn after(days: i64) -> Date {
    Date::from_day_number(today().day_number() + days).unwrap()
}

/// The defaults with cloud on.
fn on() -> VoiceSettings {
    let mut s = VoiceSettings::from_defaults().unwrap();
    s.cloud = true;
    s
}

/// The rules over the shipped table, in a run that is not hidden.
fn rules() -> Cloud<'static> {
    Cloud::new(table().unwrap(), false)
}

/// The shipped row `id`.
fn row(id: &str) -> &'static Service {
    table().unwrap().get(id).unwrap()
}

/// Shipped row `id`'s first listed model, which the tests pick.
fn first_model(id: &str) -> &'static str {
    &row(id).models[0].id
}

/// Groq's first listed model.
fn turbo() -> &'static str {
    first_model("groq")
}

/// The id of shipped row `id`'s card warning; empty for none.
fn warn_id(id: &str) -> &'static str {
    row(id).facts.warn.as_ref().map_or("", |w| w.id.as_str())
}

/// The id of `fact`, if any.
fn id_of(fact: Option<&Fact>) -> Option<&str> {
    fact.map(|f| f.id.as_str())
}

/// `OpenRouter`'s listed model every host of which keeps the voice when `keeps`, else one that a host keeping
/// nothing serves.
fn router_model(keeps: bool) -> &'static str {
    let models = &row("openrouter").models;
    models
        .iter()
        .find(|m| m.keeps == keeps)
        .map_or("", |m| m.id.as_str())
}

/// The host shipped row `id`'s card links to: its facts page's.
fn site(id: &str) -> Option<&'static str> {
    row(id).facts.source.as_ref().map(HttpsUrl::host)
}

/// The defaults with cloud on, Groq confirmed and picked for French.
fn groq_for_french() -> VoiceSettings {
    let mut s = on();
    assert!(rules().confirm(&mut s, "groq", today()));
    assert!(rules().pick(&mut s, "fr", "groq", turbo()));
    s
}

#[test]
fn no_cloud_model_is_ever_a_default() {
    let mut s = on();
    for row in table().unwrap().offered() {
        assert!(rules().confirm(&mut s, &row.id, today()), "{}", row.id);
    }
    let models = models::table().unwrap();
    let langs = models.langs().into_iter().chain(["en", "fr", "ar", "tlh"]);
    for lang in langs {
        assert!(
            rules().choice(&s, lang).is_none(),
            "{lang}: every service on, none picked"
        );
        let local = models.used(lang, &s.picks);
        assert!(local.is_none_or(|p| matches!(p, Pick::Row(_))), "{lang}");
    }
}

#[test]
fn a_service_is_off_until_the_user_confirms_its_card() {
    let mut s = on();
    let groq = row("groq");
    assert_eq!(
        rules().mark(&s, groq, true),
        ServiceMark::Off,
        "a key alone switches nothing on"
    );
    assert!(!rules().pick(&mut s, "fr", "groq", turbo()) && s.picks.is_empty());
    assert!(rules().confirm(&mut s, "groq", today()));
    let entry = &s.services["groq"];
    assert_eq!((entry.day(), entry.terms), (Some(today()), groq.terms));
    assert_eq!(rules().mark(&s, groq, false), ServiceMark::NeedsKey);
    assert_eq!(rules().mark(&s, groq, true), ServiceMark::Ready);
    assert!(rules().pick(&mut s, "fil", "groq", turbo()));
    let c = rules().choice(&s, "fil").unwrap();
    assert_eq!(
        (c.service.id.as_str(), c.code.as_str(), c.ask),
        ("groq", "tl", false)
    );
    assert!(matches!(c.model, Chosen::Listed(m) if m.id == turbo()));
    assert!(
        !rules().pick(&mut s, "tlh", "groq", turbo()),
        "the model does not know the language"
    );
    assert!(!rules().confirm(&mut s, "ovhcloud", today()), "no such row");
}

#[test]
fn edge_11_a_hidden_run_offers_nothing_and_moves_nothing_to_the_cloud() {
    let mut s = groq_for_french();
    let hidden = Cloud::new(table().unwrap(), true);
    assert!(hidden.offered(&s).is_empty() && hidden.choice(&s, "fr").is_none());
    assert_eq!(hidden.mark(&s, row("groq"), true), ServiceMark::Off);
    assert!(
        !hidden.confirm(&mut s, "custom", today()) && !hidden.pick(&mut s, "ar", "groq", turbo())
    );
    let mut off = VoiceSettings::from_defaults().unwrap();
    hidden.switch(&mut off, true);
    assert!(!off.cloud, "a hidden run never turns cloud on");
}

#[test]
fn edge_15_facts_past_facts_days_take_the_warning_tone() {
    let s = on();
    let card = |day| rules().card(&s, row("groq"), day);
    let days = i64::from(s.facts_days);
    assert!(!card(after(days)).old && card(after(days + 1)).old);
    let mut short = on();
    short.facts_days = 30;
    assert!(rules().card(&short, row("groq"), after(31)).old);
    let (groq, row_of) = (card(today()), row("groq"));
    assert_eq!(
        (groq.service, groq.read, groq.untested),
        (row_of.shown(), row_of.facts.read, row_of.untested)
    );
    assert_eq!((groq.site.as_deref(), groq.on), (site("groq"), None));
    let hours = (
        table().unwrap().light_hours(),
        table().unwrap().heavy_hours(),
    );
    assert_eq!((groq.light.hours, groq.heavy.hours), hours);
    assert_eq!(groq.free, row_of.facts.free, "the row's own allowance");
    let custom = rules().card(&s, row("custom"), after(days * 10));
    assert!(!custom.old && custom.site.is_none() && custom.service == ServiceName::Custom);
    let confirmed = groq_for_french();
    assert_eq!(
        rules().card(&confirmed, row("groq"), today()).on,
        Some(today())
    );
}

#[test]
fn edge_16_a_newer_terms_number_asks_again_and_a_no_goes_back_to_local() {
    let mut s = groq_for_french();
    s.picks.insert("nl".into(), "canary-v2-q8".into());
    assert!(rules().pick(&mut s, "ar", "groq", "whisper-large-v3"));
    let newer = parse(&TEXT.replacen("terms = 1", "terms = 2", 1)).unwrap();
    let rules = Cloud::new(&newer, false);
    let groq = newer.get("groq").unwrap();
    assert!(rules.asks(&s, groq) && rules.choice(&s, "fr").is_some_and(|c| c.ask));
    rules.turn_off(&mut s, "groq");
    assert_eq!(rules.mark(&s, groq, true), ServiceMark::Off);
    assert!(rules.choice(&s, "fr").is_none() && rules.choice(&s, "ar").is_none());
    assert_eq!(
        s.picks.keys().collect::<Vec<_>>(),
        ["nl"],
        "only the local pick stays"
    );
    let local = models::table().unwrap().used("fr", &s.picks);
    assert_eq!(
        local,
        models::table().unwrap().default_for("fr").map(Pick::Row)
    );
    assert!(rules.confirm(&mut s, "groq", today()) && !rules.asks(&s, groq));
    assert_eq!(s.services["groq"].terms, 2);
}

#[test]
fn edge_21_cloud_off_offers_nothing_and_its_picks_fall_back_to_local() {
    let mut s = groq_for_french();
    s.picks.insert("de".into(), "whisper-turbo-q5km".into());
    rules().switch(&mut s, false);
    assert!(rules().offered(&s).is_empty() && rules().choice(&s, "fr").is_none());
    assert_eq!(s.picks.keys().collect::<Vec<_>>(), ["de"]);
    assert!(s.services.contains_key("groq"), "the confirmation stays");
    rules().switch(&mut s, true);
    assert!(
        rules().choice(&s, "fr").is_none(),
        "turning cloud on brings no pick back"
    );
    let mut raw = groq_for_french();
    raw.cloud = false;
    assert!(
        rules().choice(&raw, "fr").is_none(),
        "a cloud = false in the file falls back too"
    );
    assert_eq!(raw.drop_bad(), ["picks.fr"]);
}

#[test]
fn edge_24_groq_is_listed_first() {
    let offered = rules().offered(&on());
    let ids: Vec<&str> = offered.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids.first(), Some(&"groq"));
    assert!(ids.contains(&"custom"), "{ids:?}");
}

#[test]
fn the_card_shows_the_least_a_request_bills_where_a_service_rounds_up() {
    let groq = rules().card(&on(), row("groq"), today()).bills;
    assert!(groq.is_some(), "Groq rounds a short request up");
    assert_eq!(groq, row("groq").facts.bills, "the row's own least");
    assert!(rules().card(&on(), row("custom"), today()).bills.is_none());
}

#[test]
fn a_pick_names_its_service_and_model_with_the_mark() {
    assert_eq!(
        pick_text("groq", turbo()),
        format!("groq{PICK_MARK}{}", turbo())
    );
    assert_eq!(
        cloud_pick("openrouter:openai/whisper-large-v3"),
        Some(("openrouter", "openai/whisper-large-v3"))
    );
    assert_eq!(cloud_pick("custom:llama:7b"), Some(("custom", "llama:7b")));
    // A local file whose name holds the mark, as Linux and macOS allow, stays local (P15).
    for local in [
        "canary-v2-q8",
        "my-model.gguf",
        "",
        "ollama:llama.gguf",
        ":x",
    ] {
        assert_eq!(cloud_pick(local), None, "{local}");
    }
    let test = with_rows(ROUTER);
    assert_eq!(test.pick("router-test:a:b"), Some(("router-test", "a:b")));
    assert_eq!(test.pick("groq-ish:a"), None, "only a row id parts a pick");
}

#[test]
fn a_picks_row_and_listed_model_are_found_once_for_every_reader() {
    let text = pick_text("groq", turbo());
    let (groq, model, listed) = pick_row(&text).unwrap();
    let listed = listed.map(|m| m.id.as_str());
    assert_eq!(
        (groq.id.as_str(), model, listed),
        ("groq", turbo(), Some(turbo()))
    );
    let (custom, model, listed) = pick_row("custom:llama:7b").unwrap();
    assert_eq!(
        (custom.id.as_str(), model, listed),
        ("custom", "llama:7b", None)
    );
    assert!(pick_row("canary-v2-q8").is_none(), "a local pick");
    let router = pick_text("router-test", "openai/whisper-large-v3-turbo");
    assert!(pick_row(&router).is_none(), "a row the shipped table lacks");
    let held = with_rows(ROUTER);
    let (row, model, listed) = held.pick_row(&router).unwrap();
    let listed = listed.map(|m| m.name.as_str());
    assert_eq!(
        (row.id.as_str(), model, listed),
        (
            "router-test",
            "openai/whisper-large-v3-turbo",
            Some("Whisper Large v3 Turbo")
        )
    );
}
