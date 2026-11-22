//! A service switched Off (owner, 2026-11-21): its listed picks wait, dictating nowhere, until that service's own
//! Confirm; a typed pick, declined terms (edge 16) and the `cloud` switch (edge 21) still drop them.

use super::*;

/// The service `lang`'s pick waits for, by id.
fn waits(s: &VoiceSettings, lang: &str) -> Option<String> {
    rules().waits(s, lang).map(|o| o.service.id.clone())
}

/// The service `lang` dictates with, by id.
fn used(s: &VoiceSettings, lang: &str) -> Option<String> {
    rules().choice(s, lang).map(|c| c.service.id.clone())
}

#[test]
fn a_plain_off_keeps_a_listed_pick_waiting_until_that_services_own_confirm() {
    let mut s = groq_for_french();
    rules().turn_off(&mut s, "groq");
    assert_eq!(rules().mark(&s, row("groq"), true), ServiceMark::Off);
    assert_eq!(used(&s, "fr"), None, "it dictates nowhere while Off");
    assert_eq!(waits(&s, "fr").as_deref(), Some("groq"));
    assert_eq!(waits(&s, "en"), None, "no pick, nothing waits");
    assert!(rules().confirm(&mut s, "custom", today()));
    assert_eq!(used(&s, "fr"), None, "not another service's On");
    assert!(rules().confirm(&mut s, "groq", today()));
    let back = used(&s, "fr");
    assert_eq!(back.as_deref(), Some("groq"), "its own Confirm");
    assert_eq!(waits(&s, "fr"), None);
}

#[test]
fn a_waiting_pick_waits_only_while_cloud_is_on_in_a_run_not_hidden() {
    let mut s = groq_for_french();
    rules().turn_off(&mut s, "groq");
    let hidden = Cloud::new(table().unwrap(), true);
    assert!(hidden.waits(&s, "fr").is_none(), "a hidden run");
    s.cloud = false;
    assert_eq!(waits(&s, "fr"), None, "cloud off");
    s.cloud = true;
    rules().switch(&mut s, false);
    assert!(!s.picks.contains_key("fr"), "edge 21: the switch drops it");
}

#[test]
fn off_still_drops_a_typed_pick_with_its_typed_lines() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "custom", today()));
    assert!(rules().type_address(&mut s, "custom", ADDRESS));
    assert!(rules().type_model(&mut s, "custom", "my-model"));
    assert!(rules().pick(&mut s, "en", "custom", "my-model"));
    rules().turn_off(&mut s, "custom");
    assert!(s.picks.is_empty() && waits(&s, "en").is_none());
}
