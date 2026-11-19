//! `AssemblyAI` through the cloud rules: its card carries no warning and no rule, as its European servers train on
//! nothing; a pick of Universal-3.6 Pro sends the keyboard language's own code in its own shape with no reminder, and a
//! language its set lacks cannot pick it.

use super::*;
use kx_module_api::ports::CloudShape;

/// The model the tests pick [A1][A6].
const UNIVERSAL: &str = "universal-3-6-pro";

#[test]
fn its_card_has_no_warning_or_rule_and_asks_to_switch_it_on() {
    let card = rules().card(&on(), row("assemblyai"), today());
    assert_eq!(card.service, ServiceName::Named("AssemblyAI".into()));
    assert!(
        card.warn.is_none() && !card.trains.warn,
        "no training on its EU servers [A5]"
    );
    assert!(card.rule.is_none() && card.bills.is_none());
    assert_eq!(card.site.as_deref(), Some("www.assemblyai.com"));
    assert!(card.untested && card.on.is_none(), "asks to switch it on");
}

#[test]
fn a_pick_sends_the_keyboard_languages_own_code_in_its_own_shape_with_no_reminder() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "assemblyai", today()));
    for lang in ["en", "fr", "ar"] {
        assert!(
            rules().pick(&mut s, lang, "assemblyai", UNIVERSAL),
            "{lang}"
        );
        let choice = rules().choice(&s, lang).unwrap();
        assert_eq!(choice.code, lang, "[A1]");
        let Chosen::Listed(model) = choice.model else {
            panic!("{lang}: a listed model")
        };
        assert_eq!(model.id, UNIVERSAL);
        let shape = choice.service.shape_for(Some(model), true);
        assert!(
            matches!(shape, CloudShape::Assembly { .. }),
            "{lang}: no keep field"
        );
        assert!(choice.warn.is_none(), "{lang}: nothing to remind");
    }
    assert!(
        !rules().pick(&mut s, "pl", "assemblyai", UNIVERSAL),
        "a code its pages do not name"
    );
}
