//! Gemini through the cloud rules: its card carries its row's own facts, and a pick of its first model sends that
//! model's code for the keyboard language in its own shape with the card's own reminder. A language its set lacks
//! cannot pick it.

use super::*;
use kx_module_api::ports::CloudShape;

/// The model the tests pick: the row's first.
fn transcribe() -> &'static Model {
    &row("gemini").models[0]
}

#[test]
fn its_card_carries_its_rows_facts_and_asks_to_switch_it_on() {
    card_is_its_rows("gemini");
}

#[test]
fn a_pick_sends_its_models_code_in_its_own_shape_with_the_cards_reminder() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "gemini", today()));
    for lang in ["en", "fr", "ar"] {
        let id = &transcribe().id;
        assert!(rules().pick(&mut s, lang, "gemini", id), "{lang}");
        let choice = rules().choice(&s, lang).unwrap();
        assert_eq!(
            Some(choice.code.as_str()),
            transcribe().code(lang),
            "{lang}"
        );
        let Chosen::Listed(model) = choice.model else {
            panic!("{lang}: a listed model")
        };
        assert_eq!(model.id, transcribe().id);
        let shape = choice.service.shape_for(Some(model), true);
        assert_eq!(shape, CloudShape::Gemini, "{lang}: no form, no keep field");
        let own = &row("gemini").facts.warn;
        assert_eq!(&choice.warn, own, "{lang}: the first dictation's reminder");
    }
    assert!(
        !rules().pick(
            &mut s,
            lacked("gemini", &transcribe().id),
            "gemini",
            &transcribe().id
        ),
        "a code its pages do not name"
    );
}
