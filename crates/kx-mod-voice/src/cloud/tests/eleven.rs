//! `ElevenLabs` through the cloud rules: its card carries the training warning, the training line in the warning tone
//! and the free plan's rule; a pick of Scribe sends the keyboard language's own code, and a language its set lacks
//! cannot pick it.

use super::*;
use kx_module_api::ports::CloudShape;

/// The model the tests pick: the row's first.
fn scribe() -> &'static str {
    first_model("elevenlabs")
}

#[test]
fn its_card_carries_the_warning_the_training_tone_and_the_free_plans_rule() {
    let card = rules().card(&on(), row("elevenlabs"), today());
    assert_eq!(card.service, row("elevenlabs").shown());
    let warn = card.warn.as_ref().map(|w| (w.id.as_str(), w.warn));
    assert_eq!(
        warn,
        Some((warn_id("elevenlabs"), true)),
        "the card's warning band"
    );
    assert!(card.trains.warn, "the training line takes the warning tone");
    let rule = card.rule.as_ref().map(|r| r.id.as_str());
    assert_eq!(rule, id_of(row("elevenlabs").facts.rule.as_ref()));
    assert_eq!(card.site.as_deref(), site("elevenlabs"));
    assert!(card.untested && card.on.is_none(), "asks to switch it on");
    assert!(rules().card(&on(), row("groq"), today()).rule.is_none());
}

#[test]
fn a_pick_of_scribe_sends_the_keyboard_languages_code_in_its_own_shape() {
    let mut s = on();
    assert!(rules().confirm(&mut s, "elevenlabs", today()));
    for lang in ["en", "fr", "ar"] {
        assert!(rules().pick(&mut s, lang, "elevenlabs", scribe()), "{lang}");
        let choice = rules().choice(&s, lang).unwrap();
        assert_eq!(choice.code, lang);
        let Chosen::Listed(model) = choice.model else {
            panic!("{lang}: a listed model")
        };
        assert_eq!(model.id, scribe());
        let shape = choice.service.shape_for(Some(model), true);
        assert_eq!(shape, choice.service.shape, "the row's own form, as it is");
        assert!(matches!(shape, CloudShape::Eleven(_)));
    }
    assert!(
        !rules().pick(&mut s, "de", "elevenlabs", scribe()),
        "not in its set yet"
    );
}
