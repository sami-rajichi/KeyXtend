//! The cloud rules: no cloud model is ever a default, a service is off until the user confirms its card, and edges 11
//! (a hidden run), 15 (old facts), 16 (newer terms), 21 (`cloud` off) and 24 (Groq first).

use super::*;
use crate::models::{self, Pick};
use crate::services::table;
use crate::services::tests::keyed::{raised, row_edit, row_line};
use crate::services::tests::shipped::warn_id;
use crate::services::tests::{ROUTER, TEXT, edits_text, parse, with_rows};
use kx_module_api::ports::{HttpsUrl, ServiceName, Unavailable};

mod assembly;
mod eleven;
mod gemini;
mod keep;
mod off;
mod offered;
mod parked;
mod picks;
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

/// `OpenRouter`'s listed model every host of which keeps the voice when `keeps`, else one that a host keeping
/// nothing serves.
fn router_model(keeps: bool) -> &'static str {
    let models = &row("openrouter").models;
    models
        .iter()
        .find(|m| m.keeps == keeps)
        .map_or("", |m| m.id.as_str())
}

/// A language another listed model of the table knows that shipped row `id`'s `model` lacks, so a pick of it there is
/// refused.
fn lacked(id: &str, model: &str) -> &'static str {
    let lacks = row(id).model(model).unwrap();
    let all = table().unwrap().rows().iter().flat_map(|r| &r.models);
    let mut langs = all.flat_map(|m| m.langs.keys()).map(String::as_str);
    langs.find(|l| lacks.code(l).is_none()).unwrap()
}

/// The host shipped row `id`'s card links to: its facts page's.
fn site(id: &str) -> Option<&'static str> {
    row(id).facts.source.as_ref().map(HttpsUrl::host)
}

/// Asserts shipped row `id`'s card carries its row's own facts and untested mark, and asks to switch it on.
fn card_is_its_rows(id: &str) {
    let (own, card) = (row(id), rules().card(&on(), row(id), today()));
    assert_eq!(card.service, own.shown(), "{id}");
    let facts = (&card.warn, &card.trains, &card.rule, &card.bills);
    let rows = (
        &own.facts.warn,
        &own.facts.trains,
        &own.facts.rule,
        &own.facts.bills,
    );
    assert_eq!(facts, rows, "{id}: its row's own facts");
    assert_eq!(card.site.as_deref(), site(id), "{id}");
    let asks = card.untested == own.untested && card.on.is_none();
    assert!(asks, "{id}: asks to switch it on");
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
    let read = row("groq").facts.read.day_number();
    let since = |days: i64| Date::from_day_number(read + days).unwrap();
    let days = i64::from(s.facts_days);
    assert!(!card(since(days)).old && card(since(days + 1)).old);
    let mut short = on();
    short.facts_days = 30;
    assert!(rules().card(&short, row("groq"), since(31)).old);
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
fn edge_16_a_newer_terms_number_asks_again_and_a_no_leaves_its_picks_waiting() {
    let mut s = groq_for_french();
    s.picks.insert("nl".into(), "canary-v2-q8".into());
    let other = row("groq").models.last().unwrap();
    assert!(rules().pick(&mut s, "ar", "groq", &other.id));
    let newer = parse(&raised(TEXT, "groq")).unwrap();
    let rules = Cloud::new(&newer, false);
    let groq = newer.get("groq").unwrap();
    assert!(rules.asks(&s, groq) && rules.choice(&s, "fr").is_some_and(|c| c.ask));
    rules.decline(&mut s, "groq");
    assert_eq!(rules.mark(&s, groq, true), ServiceMark::Off);
    assert!(rules.choice(&s, "fr").is_none() && rules.choice(&s, "ar").is_none());
    assert_eq!(
        s.picks.keys().collect::<Vec<_>>(),
        ["ar", "fr", "nl"],
        "ADR-0054: every pick stays"
    );
    assert!(rules.waits(&s, "fr").is_some() && rules.waits(&s, "ar").is_some());
    let local = models::table().unwrap().used("fr", &s.picks);
    assert_eq!(
        local,
        models::table().unwrap().default_for("fr").map(Pick::Row)
    );
    assert!(rules.confirm(&mut s, "groq", today()) && !rules.asks(&s, groq));
    assert_eq!(s.services["groq"].terms, groq.terms, "at the newer number");
    assert!(rules.choice(&s, "fr").is_some() && rules.choice(&s, "ar").is_some());
}

#[test]
fn edge_21_cloud_off_offers_nothing_and_its_picks_wait_for_cloud_on() {
    let mut s = groq_for_french();
    s.picks.insert("de".into(), "whisper-turbo-q5km".into());
    rules().switch(&mut s, false);
    assert!(rules().offered(&s).is_empty() && rules().choice(&s, "fr").is_none());
    assert_eq!(s.picks.keys().collect::<Vec<_>>(), ["de", "fr"], "ADR-0054");
    assert!(s.services.contains_key("groq"), "the confirmation stays");
    rules().switch(&mut s, true);
    assert!(
        rules().choice(&s, "fr").is_some(),
        "turning cloud on brings the pick back"
    );
    let mut raw = groq_for_french();
    raw.cloud = false;
    assert!(
        rules().choice(&raw, "fr").is_none(),
        "a cloud = false in the file falls back too"
    );
    assert!(raw.drop_bad().is_empty(), "ADR-0054: it waits for Cloud On");
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
