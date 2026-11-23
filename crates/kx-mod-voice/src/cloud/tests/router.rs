//! The shipped `OpenRouter` row (edge 17): its keep-nothing switch, on to start, sends its field and rules out a model
//! no host of which keeps nothing, with the reason and no move to the local model; an Other model takes a typed id,
//! sent with the keyboard's code as it is, and a typed id of such a model is ruled out too.

use super::*;
use kx_module_api::ports::{CloudShape, FormField};

/// A model id the user types, which the row does not list.
const TYPED: &str = "qwen/qwen3-asr-1.7b";

/// The defaults with cloud on and `OpenRouter` confirmed.
fn router_on() -> VoiceSettings {
    let mut s = on();
    assert!(rules().confirm(&mut s, "openrouter", today()));
    s
}

/// The fixed fields `OpenRouter`'s request carries for `model` while its switch is `keep`.
fn fixed(model: Option<&Model>, keep: bool) -> Vec<FormField> {
    match row("openrouter").shape_for(model, keep) {
        CloudShape::Form(form) => form.fixed,
        other => panic!("a form, not {other:?}"),
    }
}

#[test]
fn edge_17_the_keep_nothing_switch_rules_out_a_kept_model_with_its_reason_and_moves_nothing() {
    let mut s = router_on();
    let router = row("openrouter");
    assert!(s.services["openrouter"].keep, "on to start");
    let kept = Chosen::Listed(router.model(router_model(true)).unwrap());
    let reason = rules().unavailable(&s, router, kept);
    assert_eq!(reason, Some(Unavailable::KeepNothing));
    assert!(!rules().pick(&mut s, "fr", "openrouter", router_model(true)));
    assert!(rules().pick(&mut s, "en", "openrouter", router_model(false)));
    assert!(rules().keep(&mut s, "openrouter", false));
    assert!(rules().pick(&mut s, "fr", "openrouter", router_model(true)));
    assert!(rules().keep(&mut s, "openrouter", true));
    let c = rules().choice(&s, "fr").expect("the pick still resolves");
    assert_eq!(c.unavailable, Some(Unavailable::KeepNothing));
    assert!(s.picks.contains_key("fr"), "never a silent move to local");
    let served = rules().choice(&s, "en").unwrap();
    assert_eq!(served.unavailable, None, "a model with such a host goes");
}

#[test]
fn openrouters_request_carries_its_keep_nothing_field_only_while_its_switch_is_on() {
    let routed = row("openrouter").model(router_model(false));
    let keep: toml::Table = row_line(TEXT, "openrouter", "keep").parse().unwrap();
    let text = |key: &str| keep["keep"][key].as_str().unwrap().into();
    let zdr = FormField {
        name: text("name"),
        value: text("value"),
    };
    assert_eq!(fixed(routed, true), std::slice::from_ref(&zdr));
    assert!(fixed(routed, false).is_empty());
    assert_eq!(fixed(None, true), [zdr], "a typed model's request too");
}

#[test]
fn an_other_model_takes_a_typed_id_sent_with_the_keyboards_code_as_it_is() {
    let mut s = router_on();
    assert!(rules().type_model(&mut s, "openrouter", TYPED));
    assert!(rules().pick(&mut s, "de", "openrouter", TYPED));
    let c = rules().choice(&s, "de").unwrap();
    let got = (c.model, c.code.as_str(), c.unavailable);
    assert_eq!(got, (Chosen::Typed(TYPED), "de", None));
}

#[test]
fn a_typed_id_of_a_model_no_host_of_which_keeps_nothing_is_ruled_out_too() {
    let mut s = router_on();
    let router = row("openrouter");
    let typed = |id| rules().unavailable(&s, router, Chosen::Typed(id));
    assert_eq!(typed(router_model(true)), Some(Unavailable::KeepNothing));
    assert_eq!(typed(TYPED), None, "its hosts are checked when it is sent");
    assert!(rules().keep(&mut s, "openrouter", false));
    assert_eq!(
        rules().unavailable(&s, router, Chosen::Typed(router_model(true))),
        None
    );
}
