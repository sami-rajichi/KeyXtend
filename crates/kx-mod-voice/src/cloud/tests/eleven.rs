//! `ElevenLabs` through the cloud rules: its card carries its row's own facts, and a pick of its first model sends that
//! model's code for the keyboard language in the row's own form. A language its set lacks cannot pick it.

use super::*;
use kx_module_api::ports::CloudShape;

/// The model the tests pick: the row's first.
fn scribe() -> &'static str {
    first_model("elevenlabs")
}

#[test]
fn its_card_carries_its_rows_facts_and_asks_to_switch_it_on() {
    card_is_its_rows("elevenlabs");
}

#[test]
fn a_pick_of_scribe_sends_its_models_code_in_its_own_shape() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "elevenlabs", today()));
    for lang in ["en", "fr", "ar"] {
        assert!(rules().pick(&mut s, lang, "elevenlabs", scribe()), "{lang}");
        let choice = rules().choice(&s, lang).unwrap();
        let Chosen::Listed(model) = choice.model else {
            panic!("{lang}: a listed model")
        };
        assert_eq!(model.id, scribe());
        assert_eq!(Some(choice.code.as_str()), model.code(lang), "{lang}");
        let shape = choice.service.shape_for(Some(model), true);
        assert_eq!(shape, choice.service.shape, "the row's own form, as it is");
        assert!(matches!(shape, CloudShape::Eleven(_)));
    }
    assert!(
        !rules().pick(
            &mut s,
            lacked("elevenlabs", scribe()),
            "elevenlabs",
            scribe()
        ),
        "not in its set yet"
    );
}
