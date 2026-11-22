//! A service Off keeps its entry aside in `parked` (ADR-0054 decision 4): its typed address, typed model, keep-nothing
//! switch and every pick wait, nothing sends to it or dictates with it, and its Confirm brings it back with the new day.

use super::off::{used, waits};
use super::*;

/// A model id typed on a row's Other line.
const TYPED: &str = "my-model";
/// A model id typed on `OpenRouter`'s Other line.
const ROUTED: &str = "qwen/qwen3-asr-1.7b";

/// Custom on with its address and model typed, picked for English.
fn custom_for_english() -> VoiceSettings {
    let mut s = on();
    assert!(rules().confirm(&mut s, "custom", today()));
    assert!(rules().type_address(&mut s, "custom", ADDRESS));
    assert!(rules().type_model(&mut s, "custom", TYPED));
    assert!(rules().pick(&mut s, "en", "custom", TYPED));
    s
}

#[test]
fn off_parks_the_typed_lines_and_their_pick_and_confirm_brings_both_back() {
    let mut s = custom_for_english();
    rules().turn_off(&mut s, "custom");
    assert!(
        !s.services.contains_key("custom"),
        "`services` holds On ones only"
    );
    assert_eq!(used(&s, "en"), None, "nothing is sent while it is parked");
    assert_eq!(
        waits(&s, "en").as_deref(),
        Some("custom"),
        "the typed pick waits"
    );
    assert!(rules().confirm(&mut s, "custom", after(1)));
    let back = &s.services["custom"];
    assert_eq!(
        (back.address.as_str(), back.model.as_str()),
        (ADDRESS, TYPED)
    );
    assert_eq!(back.day(), Some(after(1)), "the new day");
    assert!(s.parked.is_empty());
    assert_eq!(used(&s, "en").as_deref(), Some("custom"));
}

#[test]
fn a_parked_entry_back_at_confirm_shows_the_cards_warning_again() {
    let mut s = custom_for_english();
    assert!(rules().warned(&mut s, "custom"));
    rules().turn_off(&mut s, "custom");
    assert!(rules().confirm(&mut s, "custom", today()));
    assert_eq!(
        s.services["custom"].warned, 0,
        "spec: again at the first dictation"
    );
}

#[test]
fn declined_terms_park_the_keep_switch_and_the_typed_model_until_confirm() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "openrouter", today()));
    assert!(rules().keep(&mut s, "openrouter", false));
    assert!(rules().type_model(&mut s, "openrouter", ROUTED));
    assert!(rules().pick(&mut s, "de", "openrouter", ROUTED));
    s.services.get_mut("openrouter").unwrap().warned = 1;
    rules().decline(&mut s, "openrouter");
    assert_eq!(waits(&s, "de").as_deref(), Some("openrouter"));
    assert!(rules().confirm(&mut s, "openrouter", today()));
    let back = &s.services["openrouter"];
    let got = (back.keep, back.model.as_str(), back.terms);
    assert_eq!(got, (false, ROUTED, row("openrouter").terms));
    assert_eq!(used(&s, "de").as_deref(), Some("openrouter"));
}

#[test]
fn nothing_offers_lists_or_sends_to_a_parked_service() {
    let mut s = custom_for_english();
    rules().turn_off(&mut s, "custom");
    let custom = row("custom");
    assert_eq!(rules().mark(&s, custom, true), ServiceMark::Off);
    assert!(!rules().asks(&s, custom));
    assert_eq!(rules().card(&s, custom, today()).on, None);
    assert!(
        rules().model_for(&s, "en", "custom", TYPED).is_none(),
        "Compare"
    );
    let listed = rules().models(&s, "en");
    assert!(
        listed.iter().all(|o| o.service.id != "custom"),
        "the Models page"
    );
    let typed = Chosen::Typed(TYPED);
    assert_eq!(rules().unavailable(&s, custom, typed), None);
    s.cloud = false;
    assert_eq!(waits(&s, "en"), None, "cloud off: the local model");
}

#[test]
fn a_parked_entry_the_row_no_longer_takes_starts_afresh_at_confirm_without_its_typed_pick() {
    let mut s = custom_for_english();
    rules().turn_off(&mut s, "custom");
    s.parked.get_mut("custom").unwrap().terms = 0;
    assert!(rules().confirm(&mut s, "custom", today()));
    let back = &s.services["custom"];
    assert!(back.address.is_empty() && back.model.is_empty());
    assert!(
        !s.picks.contains_key("en"),
        "its typed pick went with its lines"
    );
}

#[test]
fn a_service_both_on_and_parked_keeps_the_on_entry_at_confirm() {
    let mut s = custom_for_english();
    let mut old = s.services["custom"].clone();
    old.model = "old-model".into();
    s.parked.insert("custom".into(), old);
    assert!(rules().confirm(&mut s, "custom", today()));
    assert_eq!(s.services["custom"].model, TYPED);
    assert!(s.parked.is_empty());
}

#[test]
fn a_service_with_no_entry_parks_nothing() {
    let mut s = on();
    rules().turn_off(&mut s, "groq");
    assert!(s.parked.is_empty() && s.services.is_empty());
}
