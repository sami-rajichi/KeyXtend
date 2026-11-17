//! A typed row's lines: the Other model id and the address (edge 13). A typed row is neither ready nor picked until its
//! address is typed, and a typed line's change drops the picks that named it, so none comes back by itself.

use super::*;

/// Custom confirmed with cloud on.
fn custom_on() -> VoiceSettings {
    let mut s = on();
    assert!(rules().confirm(&mut s, "custom", today()));
    s
}

#[test]
fn an_other_model_is_typed_then_picked_and_its_change_drops_its_picks() {
    let mut s = on();
    let not_on = rules().type_model(&mut s, "custom", "my-model");
    assert!(!not_on, "not switched on yet");
    let mut s = custom_on();
    assert!(rules().type_address(&mut s, "custom", ADDRESS));
    assert!(!rules().type_model(&mut s, "custom", "a//b"));
    assert!(rules().type_model(&mut s, "custom", "my-model"));
    let other = rules().pick(&mut s, "en", "custom", "other-model");
    assert!(!other, "only the typed id");
    assert!(rules().pick(&mut s, "en", "custom", "my-model"));
    let c = rules().choice(&s, "en").unwrap();
    assert_eq!(
        (c.code.as_str(), c.model),
        ("en", Chosen::Typed("my-model"))
    );
    assert!(rules().type_model(&mut s, "custom", "other-model"));
    let gone = rules().choice(&s, "en").is_none() && s.picks.is_empty();
    assert!(gone, "no pick follows a new id by itself");
}

#[test]
fn a_typed_address_is_kept_only_when_https_on_a_host_name() {
    let mut s = custom_on();
    assert!(rules().confirm(&mut s, "groq", today()));
    for bad in [
        "http://stt.example.com/v1",
        "http://192.168.1.20",
        "https://192.168.1.20/v1",
        "stt",
    ] {
        assert!(!rules().type_address(&mut s, "custom", bad), "{bad}");
    }
    assert!(rules().type_address(&mut s, "custom", ADDRESS));
    assert_eq!(s.services["custom"].address, ADDRESS);
    let fixed = rules().type_address(&mut s, "groq", ADDRESS);
    assert!(!fixed, "a row with its own address takes none");
}

#[test]
fn a_typed_row_is_neither_ready_nor_picked_until_its_address_is_typed() {
    let mut s = custom_on();
    let custom = row("custom");
    assert!(rules().type_model(&mut s, "custom", "my-model"));
    assert_eq!(rules().mark(&s, custom, false), ServiceMark::NeedsKey);
    assert_eq!(
        rules().mark(&s, custom, true),
        ServiceMark::NeedsAddress,
        "a user with a key is never told to add one"
    );
    assert!(!rules().pick(&mut s, "en", "custom", "my-model") && s.picks.is_empty());
    assert!(rules().type_address(&mut s, "custom", ADDRESS));
    assert_eq!(rules().mark(&s, custom, true), ServiceMark::Ready);
    assert!(rules().pick(&mut s, "en", "custom", "my-model"));
    assert!(rules().type_address(&mut s, "custom", ""));
    assert!(s.picks.is_empty(), "a cleared address drops its picks");
    assert!(rules().type_address(&mut s, "custom", ADDRESS));
    let back = rules().choice(&s, "en");
    assert!(back.is_none(), "typing it again brings no pick back");
}
