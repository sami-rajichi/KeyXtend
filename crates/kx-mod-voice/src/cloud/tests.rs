//! The cloud rules: no cloud model is ever a default, a service is off until the user confirms its card, and edges 11
//! (a hidden run), 15 (old facts), 16 (newer terms), 21 (`cloud` off) and 24 (Groq first).

use super::*;
use crate::models::{self, Pick};
use crate::services::table;
use kx_module_api::ports::{FactValue, ServiceName};

mod props;

/// The compiled-in services table text.
const TEXT: &str = include_str!("../../services.toml");
/// Groq's turbo model.
const TURBO: &str = "whisper-large-v3-turbo";

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

/// The defaults with cloud on, Groq confirmed and picked for French.
fn groq_for_french() -> VoiceSettings {
    let mut s = on();
    assert!(rules().confirm(&mut s, "groq", today()));
    assert!(rules().pick(&mut s, "fr", "groq", TURBO));
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
    assert!(!rules().pick(&mut s, "fr", "groq", TURBO) && s.picks.is_empty());
    assert!(rules().confirm(&mut s, "groq", today()));
    let entry = &s.services["groq"];
    assert_eq!((entry.day(), entry.terms), (Some(today()), groq.terms));
    assert_eq!(rules().mark(&s, groq, false), ServiceMark::NeedsKey);
    assert_eq!(rules().mark(&s, groq, true), ServiceMark::Ready);
    assert!(rules().pick(&mut s, "fil", "groq", TURBO));
    let c = rules().choice(&s, "fil").unwrap();
    assert_eq!(
        (c.service.id.as_str(), c.code.as_str(), c.ask),
        ("groq", "tl", false)
    );
    assert!(matches!(c.model, Chosen::Listed(m) if m.id == TURBO));
    assert!(
        !rules().pick(&mut s, "tlh", "groq", TURBO),
        "the model does not know the language"
    );
    assert!(!rules().confirm(&mut s, "openai", today()), "no such row");
}

#[test]
fn an_other_model_is_typed_then_picked_and_its_change_drops_its_picks() {
    let mut s = on();
    assert!(
        !rules().type_model(&mut s, "custom", "my-model"),
        "not switched on yet"
    );
    assert!(rules().confirm(&mut s, "custom", today()));
    assert!(!rules().type_model(&mut s, "custom", "a//b"));
    assert!(rules().type_model(&mut s, "custom", "my-model"));
    assert!(
        !rules().pick(&mut s, "en", "custom", "other-model"),
        "only the typed id"
    );
    assert!(rules().pick(&mut s, "en", "custom", "my-model"));
    let c = rules().choice(&s, "en").unwrap();
    assert_eq!(
        (c.code.as_str(), c.model),
        ("en", Chosen::Typed("my-model"))
    );
    assert!(rules().type_model(&mut s, "custom", "other-model"));
    assert!(
        rules().choice(&s, "en").is_none() && s.picks.is_empty(),
        "no pick follows a new id by itself"
    );
}

#[test]
fn a_typed_address_is_kept_only_when_https_on_a_host_name() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "custom", today()) && rules().confirm(&mut s, "groq", today()));
    for bad in [
        "http://stt.example.com/v1",
        "http://192.168.1.20",
        "https://192.168.1.20/v1",
        "stt",
    ] {
        assert!(!rules().type_address(&mut s, "custom", bad), "{bad}");
    }
    let good = "https://stt.example.com/v1/audio/transcriptions";
    assert!(rules().type_address(&mut s, "custom", good));
    assert_eq!(s.services["custom"].address, good);
    assert!(
        !rules().type_address(&mut s, "groq", good),
        "a row with its own address takes none"
    );
}

#[test]
fn edge_11_a_hidden_run_offers_nothing_and_moves_nothing_to_the_cloud() {
    let mut s = groq_for_french();
    let hidden = Cloud::new(table().unwrap(), true);
    assert!(hidden.offered(&s).is_empty() && hidden.choice(&s, "fr").is_none());
    assert_eq!(hidden.mark(&s, row("groq"), true), ServiceMark::Off);
    assert!(
        !hidden.confirm(&mut s, "custom", today()) && !hidden.pick(&mut s, "ar", "groq", TURBO)
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
    let groq = card(today());
    assert_eq!(
        (groq.service, groq.read, groq.untested),
        (ServiceName::Named("Groq".into()), today(), true)
    );
    assert_eq!(
        (groq.site.as_deref(), groq.on),
        (Some("console.groq.com"), None)
    );
    assert_eq!((groq.light.hours, groq.heavy.hours), (15, 60));
    assert!(matches!(groq.free.args[0].value, FactValue::Count(8)));
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
    let newer = Services::parse(&TEXT.replacen("terms = 1", "terms = 2", 1)).unwrap();
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
    assert_eq!(ids, ["groq", "custom"]);
}

#[test]
fn a_pick_names_its_service_and_model_with_the_mark() {
    assert_eq!(pick_text("groq", TURBO), format!("groq{PICK_MARK}{TURBO}"));
    assert_eq!(
        cloud_pick("openrouter:openai/whisper-large-v3"),
        Some(("openrouter", "openai/whisper-large-v3"))
    );
    assert_eq!(cloud_pick("ollama:llama:7b"), Some(("ollama", "llama:7b")));
    for local in ["canary-v2-q8", "my-model.gguf", ""] {
        assert_eq!(cloud_pick(local), None, "{local}");
    }
}
