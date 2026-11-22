//! A service switched Off (owner, 2026-11-21) or its newer terms declined (ADR-0054): its listed picks wait, dictating
//! nowhere, until that service's own Confirm, and the `cloud` switch Off keeps every pick for Cloud On; a typed pick
//! still goes with its service's typed lines.

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
    assert!(s.picks.contains_key("fr"), "ADR-0054: the switch keeps it");
    rules().switch(&mut s, true);
    assert_eq!(waits(&s, "fr").as_deref(), Some("groq"), "waiting again");
}

#[test]
fn declined_terms_leave_the_listed_picks_waiting_for_that_services_confirm() {
    let mut s = groq_for_french();
    rules().decline(&mut s, "groq");
    assert_eq!(used(&s, "fr"), None, "it dictates nowhere while Off");
    assert_eq!(waits(&s, "fr").as_deref(), Some("groq"));
    assert!(rules().confirm(&mut s, "groq", today()));
    assert_eq!(used(&s, "fr").as_deref(), Some("groq"), "back at Confirm");
}

#[test]
fn a_confirm_over_a_stale_entry_keeps_its_listed_pick() {
    let mut s = groq_for_french();
    s.services.get_mut("groq").unwrap().terms = 0;
    assert!(rules().confirm(&mut s, "groq", today()));
    assert_eq!(used(&s, "fr").as_deref(), Some("groq"));
}

#[test]
fn the_cloud_switch_off_keeps_a_typed_pick_and_cloud_on_brings_it_back() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "custom", today()));
    assert!(rules().type_address(&mut s, "custom", ADDRESS));
    assert!(rules().type_model(&mut s, "custom", "my-model"));
    assert!(rules().pick(&mut s, "en", "custom", "my-model"));
    rules().switch(&mut s, false);
    assert_eq!(used(&s, "en"), None, "local while cloud is off");
    rules().switch(&mut s, true);
    assert_eq!(used(&s, "en").as_deref(), Some("custom"));
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
