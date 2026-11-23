//! `AssemblyAI` through the cloud rules: its card carries its row's own facts, and a pick of its first model sends that
//! model's code for the keyboard language in its own shape with the card's own reminder. A language its set lacks
//! cannot pick it.

use super::*;
use kx_module_api::ports::CloudShape;

/// The model the tests pick: the row's first.
fn universal() -> &'static str {
    first_model("assemblyai")
}

#[test]
fn its_card_carries_its_rows_facts_and_asks_to_switch_it_on() {
    card_is_its_rows("assemblyai");
}

#[test]
fn a_pick_sends_its_models_code_in_its_own_shape_with_the_cards_reminder() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "assemblyai", today()));
    let own = row("assemblyai");
    for lang in ["en", "fr", "ar"] {
        assert!(
            rules().pick(&mut s, lang, "assemblyai", universal()),
            "{lang}"
        );
        let choice = rules().choice(&s, lang).unwrap();
        let Chosen::Listed(model) = choice.model else {
            panic!("{lang}: a listed model")
        };
        assert_eq!(model.id, universal());
        assert_eq!(Some(choice.code.as_str()), model.code(lang), "{lang}");
        let shape = choice.service.shape_for(Some(model), true);
        assert!(
            matches!(shape, CloudShape::Assembly { .. }),
            "{lang}: no keep field"
        );
        assert_eq!(
            choice.warn, own.facts.warn,
            "{lang}: the card's own reminder"
        );
    }
    assert!(
        !rules().pick(
            &mut s,
            lacked("assemblyai", universal()),
            "assemblyai",
            universal()
        ),
        "a code its pages do not name"
    );
}
