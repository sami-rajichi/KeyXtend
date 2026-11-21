//! The keep-nothing switch (edge 17's plumbing), a stale entry confirmed again, and the cloud models a language may
//! pick: each switched-on service's listed models that know it, then its typed Other id.

use super::*;
use crate::services::tests::{ROUTER, with_rows};

/// The router row's model whose every host keeps the voice.
const KEPT: &str = "kept/model";
/// The router row's model with a host that keeps nothing.
const ROUTED: &str = "openai/whisper-large-v3-turbo";

/// The shipped table with the router row.
fn router() -> Services {
    with_rows(ROUTER)
}

#[test]
fn the_keep_nothing_switch_makes_a_kept_model_unavailable() {
    let table = router();
    let rules = Cloud::new(&table, false);
    let mut s = on();
    assert!(rules.confirm(&mut s, "router-test", today()));
    let row = table.get("router-test").unwrap();
    let kept = Chosen::Listed(row.model(KEPT).unwrap());
    assert!(s.services["router-test"].keep, "on to start");
    assert_eq!(
        rules.unavailable(&s, row, kept),
        Some(Unavailable::KeepNothing)
    );
    assert!(
        !rules.pick(&mut s, "en", "router-test", KEPT),
        "unavailable"
    );
    assert!(rules.pick(&mut s, "fr", "router-test", ROUTED));
    assert!(rules.keep(&mut s, "router-test", false));
    assert_eq!(rules.unavailable(&s, row, kept), None);
    assert!(rules.pick(&mut s, "en", "router-test", KEPT));
    assert!(rules.keep(&mut s, "router-test", true));
    assert!(
        rules
            .choice(&s, "fr")
            .is_some_and(|c| c.unavailable.is_none())
    );
    assert!(!rules.keep(&mut s, "groq", false), "a row with no switch");
}

#[test]
fn a_stale_entry_starts_afresh_when_confirmed_and_the_picks_that_named_it_go() {
    let mut s = on();
    let mut stale = ServiceOn::new(today(), 1);
    stale.terms = 5;
    stale.address = ADDRESS.into();
    s.services.insert("groq".into(), stale);
    s.picks.insert("fr".into(), pick_text("groq", turbo()));
    s.picks.insert("nl".into(), "canary-v2-q8".into());
    assert!(
        rules().choice(&s, "fr").is_none(),
        "a stale entry is no entry"
    );
    assert!(rules().confirm(&mut s, "groq", after(1)));
    assert_eq!(
        s.services["groq"],
        ServiceOn::new(after(1), row("groq").terms)
    );
    let left: Vec<&str> = s.picks.keys().map(String::as_str).collect();
    assert_eq!(
        left,
        ["nl"],
        "no hidden pick comes back with the fresh entry"
    );
}

#[test]
fn a_language_may_pick_each_switched_on_services_models_that_know_it() {
    let mut s = on();
    assert!(rules().models(&s, "fr").is_empty(), "nothing on yet");
    assert!(rules().confirm(&mut s, "groq", today()) && rules().confirm(&mut s, "custom", today()));
    assert!(rules().type_model(&mut s, "custom", "my-model"));
    let ids = |s: &VoiceSettings, lang| -> Vec<(String, String)> {
        let models = rules().models(s, lang);
        let pair = |m: &Offered<'_>| (m.service.id.clone(), m.id().to_owned());
        models.iter().map(pair).collect()
    };
    let fr = ids(&s, "fr");
    let want = [
        ("groq", turbo()),
        ("groq", "whisper-large-v3"),
        ("custom", "my-model"),
    ];
    assert_eq!(fr, want.map(|(a, b)| (a.to_owned(), b.to_owned())));
    assert_eq!(
        ids(&s, "tlh"),
        [("custom".to_owned(), "my-model".to_owned())]
    );
    let named = rules().models(&s, "fr")[2];
    assert_eq!(
        (named.name(), named.model),
        ("my-model", Chosen::Typed("my-model"))
    );
    let hidden = Cloud::new(table().unwrap(), true);
    assert!(
        hidden.models(&s, "fr").is_empty(),
        "a hidden run lists none"
    );
    rules().switch(&mut s, false);
    assert!(rules().models(&s, "fr").is_empty(), "cloud off lists none");
}

#[test]
fn the_keep_nothing_switch_leaves_the_pick_resolving_marked_unavailable_never_moved() {
    let table = router();
    let rules = Cloud::new(&table, false);
    let mut s = on();
    assert!(rules.confirm(&mut s, "router-test", today()));
    assert!(rules.keep(&mut s, "router-test", false));
    assert!(rules.pick(&mut s, "en", "router-test", KEPT));
    assert!(rules.keep(&mut s, "router-test", true));
    assert!(
        s.picks.contains_key("en"),
        "never a silent move to the local model"
    );
    let choice = rules.choice(&s, "en").expect("the pick still resolves");
    assert_eq!(choice.unavailable, Some(Unavailable::KeepNothing));
}
